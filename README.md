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

## See Also

Related SigmaOS **Desktop** repositories:

- [SigmaOS-shell](../SigmaOS-shell/README.md) — SigmaOS desktop shell
- [SigmaOS-shell-config](../SigmaOS-shell-config/README.md) — SigmaOS shell configuration
- [SigmaOS-hyprland-titlebars](../SigmaOS-hyprland-titlebars/README.md) — SigmaOS Hyprland titlebars
- [SigmaOS-sddm](../SigmaOS-sddm/README.md) — SigmaOS display manager theme & config
- [SigmaOS-ui](../SigmaOS-ui/README.md) — SigmaOS UI toolkit
- [SigmaOS-theme-registry](../SigmaOS-theme-registry/README.md) — SigmaOS theme registry
- [SigmaOS-theme-marketplace](../SigmaOS-theme-marketplace/README.md) — SigmaOS theme marketplace
- [SigmaOS-themes-loops](../SigmaOS-themes-loops/README.md) — SigmaOS animated theme loops
- [SigmaOS-themes-intros](../SigmaOS-themes-intros/README.md) — SigmaOS boot & login theme intros
