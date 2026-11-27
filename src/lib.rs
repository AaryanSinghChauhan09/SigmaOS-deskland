#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

/// Errors returned by the Deskland configuration engine.
#[derive(Debug, Error, PartialEq)]
pub enum DesklandError {
    #[error("monitor name cannot be empty")]
    EmptyMonitorName,

    #[error("invalid monitor scale {0}: must be greater than 0.0")]
    InvalidScale(f32),

    #[error("invalid refresh rate {0}: must be greater than 0")]
    InvalidRefreshRate(u32),

    #[error("invalid hex color '{0}': expected #RRGGBB or #RRGGBBAA")]
    InvalidColor(String),

    #[error("duplicate keybind detected: {0}")]
    DuplicateKeybind(String),

    #[error("TOML serialization error: {0}")]
    TomlSerialize(String),

    #[error("TOML deserialization error: {0}")]
    TomlDeserialize(String),
}

/// Supported Wayland/Hyprland window tiling layouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkspaceLayout {
    Dwindle,
    Master,
}

/// Modifier keys for Deskland keybinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Modifier {
    Super,
    Shift,
    Ctrl,
    Alt,
}

impl Modifier {
    fn as_hypr_str(self) -> &'static str {
        match self {
            Self::Super => "SUPER",
            Self::Shift => "SHIFT",
            Self::Ctrl => "CTRL",
            Self::Alt => "ALT",
        }
    }
}

/// Output monitor configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonitorConfig {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub refresh_rate: u32,
    pub pos_x: i32,
    pub pos_y: i32,
    pub scale: f32,
}

impl MonitorConfig {
    /// Formats a Hyprland `monitor=` directive line.
    pub fn to_hypr_line(&self) -> String {
        format!(
            "monitor={},{}x{}@{},{}x{},{}",
            self.name, self.width, self.height, self.refresh_rate, self.pos_x, self.pos_y, self.scale
        )
    }
}

/// Workspace rule binding a workspace ID to a monitor and layout.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceRule {
    pub id: u32,
    pub monitor: String,
    pub layout: WorkspaceLayout,
    pub gap_in: u32,
    pub gap_out: u32,
    pub persistent: bool,
}

/// Keybind mapping modifiers + key to a compositor dispatcher.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Keybind {
    pub modifiers: Vec<Modifier>,
    pub key: String,
    pub dispatcher: String,
    pub arg: Option<String>,
}

impl Keybind {
    /// Returns a canonical signature used for duplicate detection.
    pub fn canonical_combo(&self) -> String {
        let mut mods = self.modifiers.clone();
        mods.sort();
        mods.dedup();
        let mod_str: Vec<&str> = mods.iter().map(|m| m.as_hypr_str()).collect();
        format!("{}+{}", mod_str.join(" "), self.key.to_lowercase())
    }

    /// Formats a Hyprland `bind=` directive line.
    pub fn to_hypr_line(&self) -> String {
        let mod_str: Vec<&str> = self.modifiers.iter().map(|m| m.as_hypr_str()).collect();
        match &self.arg {
            Some(arg) if !arg.is_empty() => {
                format!("bind = {}, {}, {}, {}", mod_str.join(" "), self.key, self.dispatcher, arg)
            }
            _ => format!("bind = {}, {}, {}", mod_str.join(" "), self.key, self.dispatcher),
        }
    }
}

/// Window rule for matching application classes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowRule {
    pub class_regex: String,
    pub floating: bool,
    pub opacity: Option<f32>,
    pub workspace: Option<u32>,
}

/// Root configuration for the SigmaOS Deskland Wayland/Hyprland DE.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DesklandConfig {
    pub monitors: Vec<MonitorConfig>,
    pub workspaces: Vec<WorkspaceRule>,
    pub keybinds: Vec<Keybind>,
    pub window_rules: Vec<WindowRule>,
    pub border_size: u32,
    pub active_border_color: String,
    pub inactive_border_color: String,
    pub rounding: u32,
    pub animations_enabled: bool,
}

impl Default for DesklandConfig {
    fn default() -> Self {
        Self {
            monitors: vec![MonitorConfig {
                name: "DP-1".to_string(),
                width: 1920,
                height: 1080,
                refresh_rate: 60,
                pos_x: 0,
                pos_y: 0,
                scale: 1.0,
            }],
            workspaces: vec![WorkspaceRule {
                id: 1,
                monitor: "DP-1".to_string(),
                layout: WorkspaceLayout::Dwindle,
                gap_in: 4,
                gap_out: 8,
                persistent: true,
            }],
            keybinds: vec![
                Keybind {
                    modifiers: vec![Modifier::Super],
                    key: "Return".to_string(),
                    dispatcher: "exec".to_string(),
                    arg: Some("foot".to_string()),
                },
                Keybind {
                    modifiers: vec![Modifier::Super],
                    key: "Q".to_string(),
                    dispatcher: "killactive".to_string(),
                    arg: None,
                },
            ],
            window_rules: vec![WindowRule {
                class_regex: "^(pavucontrol)$".to_string(),
                floating: true,
                opacity: Some(0.95),
                workspace: None,
            }],
            border_size: 2,
            active_border_color: "#7aa2f7".to_string(),
            inactive_border_color: "#414868".to_string(),
            rounding: 8,
            animations_enabled: true,
        }
    }
}

fn is_valid_hex_color(color: &str) -> bool {
    let hex = match color.strip_prefix('#') {
        Some(h) => h,
        None => return false,
    };
    (hex.len() == 6 || hex.len() == 8) && hex.bytes().all(|b| b.is_ascii_hexdigit())
}

impl DesklandConfig {
    /// Validates all fields of the Deskland configuration.
    pub fn validate(&self) -> Result<(), DesklandError> {
        for monitor in &self.monitors {
            if monitor.name.trim().is_empty() {
                return Err(DesklandError::EmptyMonitorName);
            }
            if monitor.scale <= 0.0 || !monitor.scale.is_finite() {
                return Err(DesklandError::InvalidScale(monitor.scale));
            }
            if monitor.refresh_rate == 0 {
                return Err(DesklandError::InvalidRefreshRate(monitor.refresh_rate));
            }
        }

        if !is_valid_hex_color(&self.active_border_color) {
            return Err(DesklandError::InvalidColor(self.active_border_color.clone()));
        }
        if !is_valid_hex_color(&self.inactive_border_color) {
            return Err(DesklandError::InvalidColor(self.inactive_border_color.clone()));
        }

        let mut seen_binds = HashSet::new();
        for bind in &self.keybinds {
            let combo = bind.canonical_combo();
            if !seen_binds.insert(combo.clone()) {
                return Err(DesklandError::DuplicateKeybind(combo));
            }
        }

        Ok(())
    }

    /// Serializes the configuration to TOML.
    pub fn to_toml(&self) -> Result<String, DesklandError> {
        toml::to_string_pretty(self).map_err(|e| DesklandError::TomlSerialize(e.to_string()))
    }

    /// Deserializes and validates a configuration from TOML.
    pub fn from_toml(input: &str) -> Result<Self, DesklandError> {
        let cfg: Self =
            toml::from_str(input).map_err(|e| DesklandError::TomlDeserialize(e.to_string()))?;
        cfg.validate()?;
        Ok(cfg)
    }

    /// Renders a complete Hyprland-compatible configuration string.
    pub fn render_hyprland_conf(&self) -> Result<String, DesklandError> {
        self.validate()?;
        let mut out = String::from("# Generated by SigmaOS-deskland\n\n");

        for m in &self.monitors {
            out.push_str(&m.to_hypr_line());
            out.push('\n');
        }
        out.push('\n');

        let active_hex = self.active_border_color.trim_start_matches('#');
        let inactive_hex = self.inactive_border_color.trim_start_matches('#');
        out.push_str("general {\n");
        out.push_str(&format!("    border_size = {}\n", self.border_size));
        out.push_str(&format!("    col.active_border = rgb({})\n", active_hex));
        out.push_str(&format!("    col.inactive_border = rgb({})\n", inactive_hex));
        out.push_str("}\n\n");

        out.push_str("decoration {\n");
        out.push_str(&format!("    rounding = {}\n", self.rounding));
        out.push_str("}\n\n");

        out.push_str("animations {\n");
        out.push_str(&format!(
            "    enabled = {}\n",
            if self.animations_enabled { "yes" } else { "no" }
        ));
        out.push_str("}\n\n");

        for ws in &self.workspaces {
            let persistent = if ws.persistent { ", persistent:true" } else { "" };
            out.push_str(&format!(
                "workspace = {}, monitor:{}{}\n",
                ws.id, ws.monitor, persistent
            ));
        }
        out.push('\n');

        for bind in &self.keybinds {
            out.push_str(&bind.to_hypr_line());
            out.push('\n');
        }

        for rule in &self.window_rules {
            if rule.floating {
                out.push_str(&format!("windowrulev2 = float, class:{}\n", rule.class_regex));
            }
            if let Some(op) = rule.opacity {
                out.push_str(&format!(
                    "windowrulev2 = opacity {}, class:{}\n",
                    op, rule.class_regex
                ));
            }
            if let Some(ws) = rule.workspace {
                out.push_str(&format!(
                    "windowrulev2 = workspace {}, class:{}\n",
                    ws, rule.class_regex
                ));
            }
        }

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_validates_and_renders_aaa() {
        // Arrange
        let config = DesklandConfig::default();

        // Act
        let validation = config.validate();
        let rendered = config.render_hyprland_conf();

        // Assert
        assert!(validation.is_ok());
        let conf_text = rendered.expect("rendered config");
        assert!(conf_text.contains("monitor=DP-1,1920x1080@60,0x0,1"));
        assert!(conf_text.contains("col.active_border = rgb(7aa2f7)"));
        assert!(conf_text.contains("bind = SUPER, Return, exec, foot"));
    }

    #[test]
    fn test_duplicate_keybind_rejected_aaa() {
        // Arrange
        let mut config = DesklandConfig::default();
        config.keybinds.push(Keybind {
            modifiers: vec![Modifier::Super],
            key: "q".to_string(),
            dispatcher: "exit".to_string(),
            arg: None,
        });

        // Act
        let result = config.validate();

        // Assert
        assert_eq!(
            result,
            Err(DesklandError::DuplicateKeybind("SUPER+q".to_string()))
        );
    }

    #[test]
    fn test_toml_roundtrip_aaa() {
        // Arrange
        let config = DesklandConfig::default();

        // Act
        let toml_str = config.to_toml().expect("serialize to toml");
        let parsed = DesklandConfig::from_toml(&toml_str).expect("deserialize from toml");

        // Assert
        assert_eq!(config, parsed);
    }

    #[test]
    fn test_invalid_hex_color_rejected_aaa() {
        // Arrange
        let mut config = DesklandConfig::default();
        config.active_border_color = "not-a-color".to_string();

        // Act
        let result = config.validate();

        // Assert
        assert_eq!(
            result,
            Err(DesklandError::InvalidColor("not-a-color".to_string()))
        );
    }
}
