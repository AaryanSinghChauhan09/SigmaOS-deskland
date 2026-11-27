# SigmaOS-deskland

Desktop environment built on Wayland/Hyprland with a modular Rust configuration engine for SigmaOS.

## Features

- Declarative monitor, workspace, keybind, and window rule configuration
- Configuration validation (hex colors, scale factors, duplicate keybind detection)
- TOML serialization/deserialization
- Compositor configuration rendering

## Usage

```bash
cargo run -- init-default
cargo run -- render
cargo test
```

## License

GPL-3.0-or-later
