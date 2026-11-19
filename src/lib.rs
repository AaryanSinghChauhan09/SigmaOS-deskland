//! SigmaOS Deskland — Web browser-based desktop environment clone.
//!
//! Provides a complete in-browser window manager, desktop surface, taskbar,
//! application launcher, and IPC integration for SigmaOS web applications.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use thiserror::Error;

/// Error conditions for Deskland desktop operations.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DesklandError {
    /// Window was not found in registry.
    #[error("window with id {0} not found")]
    WindowNotFound(u32),
    /// Invalid window dimensions or coordinates.
    #[error("invalid dimensions: width={0}, height={1}")]
    InvalidDimensions(u32, u32),
    /// Application not registered in launcher.
    #[error("application with id '{0}' not registered")]
    AppNotFound(String),
    /// Operation invalid in current window state.
    #[error("invalid operation '{0}' for window state")]
    InvalidState(&'static str),
}

/// Convenience result alias for Deskland operations.
pub type DesklandResult<T> = Result<T, DesklandError>;

/// Display state of a window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowState {
    /// Standard floating window with defined position and dimensions.
    Normal,
    /// Window minimized to taskbar (invisible on desktop surface).
    Minimized,
    /// Window occupying the entire available workspace.
    Maximized,
}

/// 2D rectangle specifying window position and dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowRect {
    /// X coordinate in pixels from workspace top-left.
    pub x: i32,
    /// Y coordinate in pixels from workspace top-left.
    pub y: i32,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

impl WindowRect {
    /// Construct a new rectangle with bounds validation.
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> DesklandResult<Self> {
        if width == 0 || height == 0 {
            return Err(DesklandError::InvalidDimensions(width, height));
        }
        Ok(Self { x, y, width, height })
    }
}

/// A desktop application window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    /// Unique integer window identifier.
    pub id: u32,
    /// Window title bar text.
    pub title: String,
    /// Identifier of the running application.
    pub app_id: String,
    /// Current window bounds.
    pub rect: WindowRect,
    /// Saved bounds before maximization.
    pub saved_rect: Option<WindowRect>,
    /// Current window display state.
    pub state: WindowState,
    /// Focus / depth order (higher is frontmost).
    pub z_index: u32,
    /// Whether this window currently has keyboard and mouse focus.
    pub focused: bool,
}

/// Desktop theme palettes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopTheme {
    /// Sovereign Dark flagship aesthetic.
    SovereignDark,
    /// Catppuccin Mocha aesthetic.
    CatppuccinMocha,
    /// Mint Emerald aesthetic.
    MintEmerald,
    /// Arch Cyan minimalist aesthetic.
    ArchCyan,
}

impl DesktopTheme {
    /// Get the primary background color in hexadecimal.
    pub fn background_hex(&self) -> &'static str {
        match self {
            Self::SovereignDark => "#0f141c",
            Self::CatppuccinMocha => "#1e1e2e",
            Self::MintEmerald => "#1a221f",
            Self::ArchCyan => "#171a1f",
        }
    }

    /// Get the accent color in hexadecimal.
    pub fn accent_hex(&self) -> &'static str {
        match self {
            Self::SovereignDark => "#5b9bf8",
            Self::CatppuccinMocha => "#cba6f7",
            Self::MintEmerald => "#2ecc71",
            Self::ArchCyan => "#1793d1",
        }
    }
}

/// Deskland Window Manager responsible for window lifecycle, geometry, and focus.
#[derive(Debug, Clone)]
pub struct WindowManager {
    windows: Vec<Window>,
    next_id: u32,
    workspace_width: u32,
    workspace_height: u32,
}

impl WindowManager {
    /// Create a new WindowManager with defined display workspace dimensions.
    pub fn new(workspace_width: u32, workspace_height: u32) -> Self {
        Self {
            windows: Vec::new(),
            next_id: 1,
            workspace_width,
            workspace_height,
        }
    }

    /// Open and focus a new window with specified title, app_id, and dimensions.
    pub fn create_window(
        &mut self,
        title: impl Into<String>,
        app_id: impl Into<String>,
        rect: WindowRect,
    ) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        // Lower existing windows
        for w in &mut self.windows {
            w.focused = false;
        }

        let max_z = self.windows.iter().map(|w| w.z_index).max().unwrap_or(0);

        let window = Window {
            id,
            title: title.into(),
            app_id: app_id.into(),
            rect,
            saved_rect: None,
            state: WindowState::Normal,
            z_index: max_z + 1,
            focused: true,
        };

        self.windows.push(window);
        id
    }

    /// Get reference to a window by ID.
    pub fn get_window(&self, id: u32) -> Option<&Window> {
        self.windows.iter().find(|w| w.id == id)
    }

    /// Get mutable reference to a window by ID.
    pub fn get_window_mut(&mut self, id: u32) -> Option<&mut Window> {
        self.windows.iter_mut().find(|w| w.id == id)
    }

    /// Close and remove a window.
    pub fn close_window(&mut self, id: u32) -> DesklandResult<()> {
        let pos = self
            .windows
            .iter()
            .position(|w| w.id == id)
            .ok_or(DesklandError::WindowNotFound(id))?;

        self.windows.remove(pos);

        // Refocus next highest window if any
        if let Some(front) = self
            .windows
            .iter_mut()
            .filter(|w| w.state != WindowState::Minimized)
            .max_by_key(|w| w.z_index)
        {
            front.focused = true;
        }

        Ok(())
    }

    /// Focus a window and bring it to the front of the z-stack.
    pub fn focus_window(&mut self, id: u32) -> DesklandResult<()> {
        let target_exists = self.windows.iter().any(|w| w.id == id);
        if !target_exists {
            return Err(DesklandError::WindowNotFound(id));
        }

        let max_z = self.windows.iter().map(|w| w.z_index).max().unwrap_or(0);

        for w in &mut self.windows {
            if w.id == id {
                w.z_index = max_z + 1;
                w.focused = true;
                if w.state == WindowState::Minimized {
                    w.state = WindowState::Normal;
                }
            } else {
                w.focused = false;
            }
        }

        Ok(())
    }

    /// Minimize a window to the taskbar.
    pub fn minimize_window(&mut self, id: u32) -> DesklandResult<()> {
        let w = self
            .get_window_mut(id)
            .ok_or(DesklandError::WindowNotFound(id))?;

        w.state = WindowState::Minimized;
        w.focused = false;

        // Focus next highest non-minimized window
        if let Some(front) = self
            .windows
            .iter_mut()
            .filter(|w| w.id != id && w.state != WindowState::Minimized)
            .max_by_key(|w| w.z_index)
        {
            front.focused = true;
        }

        Ok(())
    }

    /// Maximize a window to occupy full workspace bounds.
    pub fn maximize_window(&mut self, id: u32) -> DesklandResult<()> {
        let ws_w = self.workspace_width;
        let ws_h = self.workspace_height;
        let w = self
            .get_window_mut(id)
            .ok_or(DesklandError::WindowNotFound(id))?;

        if w.state == WindowState::Maximized {
            return Ok(());
        }

        w.saved_rect = Some(w.rect);
        w.rect = WindowRect {
            x: 0,
            y: 0,
            width: ws_w,
            height: ws_h,
        };
        w.state = WindowState::Maximized;
        Ok(())
    }

    /// Restore a maximized or minimized window to its previous floating geometry.
    pub fn restore_window(&mut self, id: u32) -> DesklandResult<()> {
        let w = self
            .get_window_mut(id)
            .ok_or(DesklandError::WindowNotFound(id))?;

        if let Some(saved) = w.saved_rect {
            w.rect = saved;
            w.saved_rect = None;
        }
        w.state = WindowState::Normal;
        w.focused = true;
        Ok(())
    }

    /// Snap a window to the left half of the display workspace.
    pub fn snap_left(&mut self, id: u32) -> DesklandResult<()> {
        let half_w = self.workspace_width / 2;
        let h = self.workspace_height;
        let w = self
            .get_window_mut(id)
            .ok_or(DesklandError::WindowNotFound(id))?;

        w.saved_rect = Some(w.rect);
        w.rect = WindowRect {
            x: 0,
            y: 0,
            width: half_w,
            height: h,
        };
        w.state = WindowState::Normal;
        Ok(())
    }

    /// Snap a window to the right half of the display workspace.
    pub fn snap_right(&mut self, id: u32) -> DesklandResult<()> {
        let half_w = self.workspace_width / 2;
        let h = self.workspace_height;
        let w = self
            .get_window_mut(id)
            .ok_or(DesklandError::WindowNotFound(id))?;

        w.saved_rect = Some(w.rect);
        w.rect = WindowRect {
            x: half_w as i32,
            y: 0,
            width: half_w,
            height: h,
        };
        w.state = WindowState::Normal;
        Ok(())
    }

    /// Total count of open windows.
    pub fn window_count(&self) -> usize {
        self.windows.len()
    }

    /// Return all active windows.
    pub fn windows(&self) -> &[Window] {
        &self.windows
    }

    /// Returns the currently focused window, if any.
    pub fn active_window(&self) -> Option<&Window> {
        self.windows.iter().find(|w| w.focused)
    }
}

/// An application launch entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppEntry {
    /// Unique app identifier (e.g. "betteroffice-docs").
    pub id: String,
    /// Display name shown in menu and title bars.
    pub name: String,
    /// Application category.
    pub category: String,
    /// Icon name or CSS class.
    pub icon: String,
    /// Target web URL or route.
    pub route: String,
}

/// Application launcher registry cataloging available web clones.
#[derive(Debug, Clone, Default)]
pub struct AppLauncher {
    apps: Vec<AppEntry>,
}

impl AppLauncher {
    /// Create and initialize default catalog of SigmaOS web applications.
    pub fn new() -> Self {
        let mut launcher = Self { apps: Vec::new() };
        launcher.register_defaults();
        launcher
    }

    /// Register default suite of web browser clones.
    fn register_defaults(&mut self) {
        // BetterOffice Suite
        self.register("betteroffice-diagrams", "BetterOffice Diagrams", "Office", "diagrams", "/apps/betteroffice/diagrams");
        self.register("betteroffice-docs", "BetterOffice Documents", "Office", "document", "/apps/betteroffice/docs");
        self.register("betteroffice-sheets", "BetterOffice Sheets", "Office", "table", "/apps/betteroffice/sheets");
        self.register("betteroffice-slides", "BetterOffice Slides", "Office", "presentation", "/apps/betteroffice/slides");

        // GenOffice Suite
        self.register("genoffice-docs", "GenOffice Docs", "Office", "file-text", "/apps/genoffice/docs");
        self.register("genoffice-sheets", "GenOffice Sheets", "Office", "grid", "/apps/genoffice/sheets");
        self.register("genoffice-pdf", "GenOffice PDF", "Office", "file", "/apps/genoffice/pdf");

        // Craft Creative Suite
        self.register("craft-3d", "Craft 3D Studio", "Creative", "box", "/apps/craft/3d");
        self.register("craft-art", "Craft Digital Art", "Creative", "palette", "/apps/craft/art");
        self.register("craft-cad", "Craft CAD Modeler", "Creative", "cpu", "/apps/craft/cad");
        self.register("craft-photo", "Craft Photo Editor", "Creative", "image", "/apps/craft/photo");
        self.register("craft-vector", "Craft Vector Design", "Creative", "pen-tool", "/apps/craft/vector");

        // DOS Emulation Suite
        self.register("dos-free", "FreeDOS Environment", "Emulation", "terminal", "/apps/dos/free");
        self.register("dos-ms", "MS-DOS Environment", "Emulation", "terminal", "/apps/dos/ms");
        self.register("dos-basic", "BASIC Interactive Interpreter", "Emulation", "code", "/apps/dos/basic");

        // Data Science & RDBMS
        self.register("rdbms-duckdb", "DuckDB Analytics Studio", "Data", "database", "/apps/rdbms/duckdb");
        self.register("rdbms-postgresql", "PostgreSQL Web Console", "Data", "server", "/apps/rdbms/postgresql");
        self.register("tools-spark", "Apache Spark Cluster UI", "Data", "activity", "/apps/tools/spark");
    }

    /// Register a new application in the catalog.
    pub fn register(
        &mut self,
        id: impl Into<String>,
        name: impl Into<String>,
        category: impl Into<String>,
        icon: impl Into<String>,
        route: impl Into<String>,
    ) {
        self.apps.push(AppEntry {
            id: id.into(),
            name: name.into(),
            category: category.into(),
            icon: icon.into(),
            route: route.into(),
        });
    }

    /// Search registered applications by case-insensitive name, id, or category query.
    pub fn search(&self, query: &str) -> Vec<&AppEntry> {
        let q = query.to_lowercase();
        self.apps
            .iter()
            .filter(|app| {
                app.id.to_lowercase().contains(&q)
                    || app.name.to_lowercase().contains(&q)
                    || app.category.to_lowercase().contains(&q)
            })
            .collect()
    }

    /// Get all applications under a specific category.
    pub fn get_by_category(&self, category: &str) -> Vec<&AppEntry> {
        self.apps.iter().filter(|app| app.category.eq_ignore_ascii_case(category)).collect()
    }

    /// Get an application by exact ID.
    pub fn get(&self, id: &str) -> Option<&AppEntry> {
        self.apps.iter().find(|app| app.id == id)
    }

    /// Total count of registered applications.
    pub fn app_count(&self) -> usize {
        self.apps.len()
    }
}

/// A desktop surface shortcut icon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopIcon {
    /// Identifier.
    pub id: String,
    /// Label displayed under icon.
    pub label: String,
    /// Associated application ID.
    pub app_id: String,
    /// Grid column (0-indexed).
    pub col: u32,
    /// Grid row (0-indexed).
    pub row: u32,
}

/// Desktop surface managing wallpaper, theme, and desktop icons.
#[derive(Debug, Clone)]
pub struct DesktopSurface {
    theme: DesktopTheme,
    icons: Vec<DesktopIcon>,
}

impl DesktopSurface {
    /// Create a desktop surface with default theme and icons.
    pub fn new(theme: DesktopTheme) -> Self {
        let mut surface = Self {
            theme,
            icons: Vec::new(),
        };
        surface.populate_default_icons();
        surface
    }

    fn populate_default_icons(&mut self) {
        self.add_icon("betteroffice-docs", "BetterOffice Docs", "betteroffice-docs", 0, 0);
        self.add_icon("betteroffice-sheets", "BetterOffice Sheets", "betteroffice-sheets", 0, 1);
        self.add_icon("craft-art", "Craft Art", "craft-art", 0, 2);
        self.add_icon("dos-free", "FreeDOS", "dos-free", 0, 3);
        self.add_icon("rdbms-duckdb", "DuckDB Studio", "rdbms-duckdb", 0, 4);
    }

    /// Add an icon to the desktop grid.
    pub fn add_icon(
        &mut self,
        id: impl Into<String>,
        label: impl Into<String>,
        app_id: impl Into<String>,
        col: u32,
        row: u32,
    ) {
        self.icons.push(DesktopIcon {
            id: id.into(),
            label: label.into(),
            app_id: app_id.into(),
            col,
            row,
        });
    }

    /// Get current desktop theme.
    pub fn theme(&self) -> DesktopTheme {
        self.theme
    }

    /// Set desktop theme.
    pub fn set_theme(&mut self, theme: DesktopTheme) {
        self.theme = theme;
    }

    /// Get all desktop icons.
    pub fn icons(&self) -> &[DesktopIcon] {
        &self.icons
    }
}

/// Deskland Desktop Environment root orchestrator.
#[derive(Debug, Clone)]
pub struct Deskland {
    /// Window manager subsystem.
    pub wm: WindowManager,
    /// Application launcher catalog.
    pub launcher: AppLauncher,
    /// Desktop surface.
    pub surface: DesktopSurface,
    /// Start menu opened state.
    pub start_menu_open: bool,
}

impl Deskland {
    /// Create a new Deskland environment for specified workspace dimensions.
    pub fn new(width: u32, height: u32, theme: DesktopTheme) -> Self {
        Self {
            wm: WindowManager::new(width, height),
            launcher: AppLauncher::new(),
            surface: DesktopSurface::new(theme),
            start_menu_open: false,
        }
    }

    /// Launch an application by ID into a new window.
    pub fn launch_app(&mut self, app_id: &str) -> DesklandResult<u32> {
        let app = self
            .launcher
            .get(app_id)
            .ok_or_else(|| DesklandError::AppNotFound(app_id.to_string()))?
            .clone();

        let count = self.wm.window_count() as i32;
        let cascade_offset = (count * 30) % 200;

        let rect = WindowRect::new(80 + cascade_offset, 60 + cascade_offset, 800, 500)?;
        let window_id = self.wm.create_window(app.name, app.id, rect);
        self.start_menu_open = false;
        Ok(window_id)
    }

    /// Toggle start menu visibility.
    pub fn toggle_start_menu(&mut self) {
        self.start_menu_open = !self.start_menu_open;
    }

    /// Generate initial HTML/CSS document bundle for web browser rendering.
    pub fn render_html(&self) -> String {
        let bg = self.surface.theme().background_hex();
        let accent = self.surface.theme().accent_hex();
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>SigmaOS Deskland</title>
    <style>
        body {{ margin: 0; background-color: {bg}; color: #ffffff; font-family: 'JetBrains Mono', sans-serif; overflow: hidden; }}
        .panel {{ position: fixed; bottom: 0; left: 0; right: 0; height: 48px; background: rgba(15, 20, 28, 0.85); backdrop-filter: blur(10px); border-top: 1px solid {accent}; display: flex; align-items: center; padding: 0 12px; }}
        .start-btn {{ background: {accent}; color: #000; font-weight: bold; border: none; padding: 6px 14px; border-radius: 6px; cursor: pointer; }}
    </style>
</head>
<body>
    <div id="desktop-surface"></div>
    <div class="panel">
        <button class="start-btn">SigmaOS</button>
    </div>
</body>
</html>"#
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_creation_and_focus_aaa() {
        // Arrange
        let mut wm = WindowManager::new(1920, 1080);
        let rect = WindowRect::new(100, 100, 800, 600).unwrap();

        // Act
        let id = wm.create_window("Docs", "betteroffice-docs", rect);
        let win = wm.get_window(id).unwrap();

        // Assert
        assert_eq!(win.id, id);
        assert_eq!(win.title, "Docs");
        assert!(win.focused);
        assert_eq!(win.state, WindowState::Normal);
    }

    #[test]
    fn test_minimize_and_restore_window_aaa() {
        // Arrange
        let mut wm = WindowManager::new(1920, 1080);
        let rect = WindowRect::new(50, 50, 600, 400).unwrap();
        let id = wm.create_window("Sheets", "betteroffice-sheets", rect);

        // Act - Minimize
        wm.minimize_window(id).unwrap();
        let win_min = wm.get_window(id).unwrap();

        // Assert - Minimized
        assert_eq!(win_min.state, WindowState::Minimized);
        assert!(!win_min.focused);

        // Act - Restore
        wm.restore_window(id).unwrap();
        let win_restored = wm.get_window(id).unwrap();

        // Assert - Restored
        assert_eq!(win_restored.state, WindowState::Normal);
        assert!(win_restored.focused);
    }

    #[test]
    fn test_maximize_and_restore_window_aaa() {
        // Arrange
        let mut wm = WindowManager::new(1920, 1080);
        let orig_rect = WindowRect::new(100, 100, 640, 480).unwrap();
        let id = wm.create_window("Craft", "craft-art", orig_rect);

        // Act - Maximize
        wm.maximize_window(id).unwrap();
        let win_max = wm.get_window(id).unwrap();

        // Assert - Maximized to full workspace
        assert_eq!(win_max.state, WindowState::Maximized);
        assert_eq!(win_max.rect.width, 1920);
        assert_eq!(win_max.rect.height, 1080);

        // Act - Restore
        wm.restore_window(id).unwrap();
        let win_restored = wm.get_window(id).unwrap();

        // Assert - Back to original geometry
        assert_eq!(win_restored.state, WindowState::Normal);
        assert_eq!(win_restored.rect, orig_rect);
    }

    #[test]
    fn test_window_snap_tiling_aaa() {
        // Arrange
        let mut wm = WindowManager::new(1920, 1080);
        let rect = WindowRect::new(200, 200, 500, 400).unwrap();
        let id1 = wm.create_window("Left", "betteroffice-docs", rect);
        let id2 = wm.create_window("Right", "craft-3d", rect);

        // Act
        wm.snap_left(id1).unwrap();
        wm.snap_right(id2).unwrap();

        let w1 = wm.get_window(id1).unwrap();
        let w2 = wm.get_window(id2).unwrap();

        // Assert
        assert_eq!(w1.rect.x, 0);
        assert_eq!(w1.rect.width, 960);
        assert_eq!(w2.rect.x, 960);
        assert_eq!(w2.rect.width, 960);
    }

    #[test]
    fn test_focus_and_z_stack_ordering_aaa() {
        // Arrange
        let mut wm = WindowManager::new(1920, 1080);
        let rect = WindowRect::new(50, 50, 400, 300).unwrap();
        let id1 = wm.create_window("Win 1", "app1", rect);
        let id2 = wm.create_window("Win 2", "app2", rect);

        // Assert initial z-order (id2 is frontmost)
        assert!(wm.get_window(id2).unwrap().z_index > wm.get_window(id1).unwrap().z_index);

        // Act - Refocus id1
        wm.focus_window(id1).unwrap();

        // Assert new z-order (id1 is now frontmost)
        let w1 = wm.get_window(id1).unwrap();
        let w2 = wm.get_window(id2).unwrap();
        assert!(w1.z_index > w2.z_index);
        assert!(w1.focused);
        assert!(!w2.focused);
    }

    #[test]
    fn test_app_launcher_search_and_categories_aaa() {
        // Arrange
        let launcher = AppLauncher::new();

        // Act
        let office_apps = launcher.get_by_category("Office");
        let query_docs = launcher.search("docs");

        // Assert
        assert!(!office_apps.is_empty());
        assert!(query_docs.iter().any(|app| app.id == "betteroffice-docs"));
        assert!(query_docs.iter().any(|app| app.id == "genoffice-docs"));
    }

    #[test]
    fn test_deskland_app_launching_lifecycle_aaa() {
        // Arrange
        let mut deskland = Deskland::new(1920, 1080, DesktopTheme::SovereignDark);

        // Act
        let win_id = deskland.launch_app("betteroffice-docs").unwrap();

        // Assert
        assert_eq!(deskland.wm.window_count(), 1);
        let win = deskland.wm.get_window(win_id).unwrap();
        assert_eq!(win.app_id, "betteroffice-docs");
        assert!(win.focused);
    }

    #[test]
    fn test_theme_properties_and_html_rendering_aaa() {
        // Arrange
        let deskland = Deskland::new(1920, 1080, DesktopTheme::SovereignDark);

        // Act
        let html = deskland.render_html();

        // Assert
        assert!(html.contains("SigmaOS Deskland"));
        assert!(html.contains("#0f141c"));
        assert!(html.contains("#5b9bf8"));
    }

    #[test]
    fn test_invalid_dimensions_error_aaa() {
        // Arrange & Act
        let res = WindowRect::new(0, 0, 0, 100);

        // Assert
        assert_eq!(res, Err(DesklandError::InvalidDimensions(0, 100)));
    }
}
