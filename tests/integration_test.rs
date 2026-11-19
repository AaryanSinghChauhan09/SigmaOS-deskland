use sigmaos_deskland::{Deskland, DesktopTheme, WindowState};

#[test]
fn test_deskland_multi_window_management_aaa() {
    // Arrange
    let mut deskland = Deskland::new(1920, 1080, DesktopTheme::SovereignDark);

    // Act
    let id_docs = deskland.launch_app("betteroffice-docs").unwrap();
    let id_sheets = deskland.launch_app("betteroffice-sheets").unwrap();
    let id_cad = deskland.launch_app("craft-cad").unwrap();

    // Assert
    assert_eq!(deskland.wm.window_count(), 3);
    assert_eq!(deskland.wm.active_window().unwrap().id, id_cad);

    // Act - Minimize cad and focus docs
    deskland.wm.minimize_window(id_cad).unwrap();
    assert_eq!(deskland.wm.get_window(id_cad).unwrap().state, WindowState::Minimized);

    deskland.wm.focus_window(id_docs).unwrap();
    assert_eq!(deskland.wm.active_window().unwrap().id, id_docs);

    // Act - Close sheets
    deskland.wm.close_window(id_sheets).unwrap();
    assert_eq!(deskland.wm.window_count(), 2);
}

#[test]
fn test_deskland_surface_and_theme_switching_aaa() {
    // Arrange
    let mut deskland = Deskland::new(1920, 1080, DesktopTheme::MintEmerald);
    assert_eq!(deskland.surface.theme(), DesktopTheme::MintEmerald);

    // Act
    deskland.surface.set_theme(DesktopTheme::CatppuccinMocha);

    // Assert
    assert_eq!(deskland.surface.theme(), DesktopTheme::CatppuccinMocha);
    let html = deskland.render_html();
    assert!(html.contains("#1e1e2e"));
}

#[test]
fn test_deskland_tiling_workflow_aaa() {
    // Arrange
    let mut deskland = Deskland::new(2560, 1440, DesktopTheme::ArchCyan);
    let win1 = deskland.launch_app("betteroffice-docs").unwrap();
    let win2 = deskland.launch_app("craft-3d").unwrap();

    // Act
    deskland.wm.snap_left(win1).unwrap();
    deskland.wm.snap_right(win2).unwrap();

    // Assert
    let w1 = deskland.wm.get_window(win1).unwrap();
    let w2 = deskland.wm.get_window(win2).unwrap();
    assert_eq!(w1.rect.x, 0);
    assert_eq!(w1.rect.width, 1280);
    assert_eq!(w2.rect.x, 1280);
    assert_eq!(w2.rect.width, 1280);
}
