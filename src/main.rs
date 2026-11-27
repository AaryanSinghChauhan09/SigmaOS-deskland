use sigmaos_deskland::DesklandConfig;
use std::env;
use std::fs;
use std::process;

fn print_usage() {
    println!("sigmaos-deskland - SigmaOS Wayland/Hyprland DE Config Engine");
    println!("Usage:");
    println!("  sigmaos-deskland init-default       Print default TOML configuration");
    println!("  sigmaos-deskland render [file.toml] Render Hyprland configuration");
    println!("  sigmaos-deskland validate <file>    Validate a TOML configuration file");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let cmd = args.get(1).map(String::as_str).unwrap_or("help");

    match cmd {
        "init-default" => {
            let cfg = DesklandConfig::default();
            match cfg.to_toml() {
                Ok(text) => print!("{}", text),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            }
        }
        "render" => {
            let cfg = if let Some(path) = args.get(2) {
                let raw = fs::read_to_string(path).unwrap_or_else(|e| {
                    eprintln!("Failed to read {}: {}", path, e);
                    process::exit(1);
                });
                DesklandConfig::from_toml(&raw).unwrap_or_else(|e| {
                    eprintln!("Invalid config: {}", e);
                    process::exit(1);
                })
            } else {
                DesklandConfig::default()
            };
            match cfg.render_hyprland_conf() {
                Ok(out) => print!("{}", out),
                Err(e) => {
                    eprintln!("Render error: {}", e);
                    process::exit(1);
                }
            }
        }
        "validate" => {
            let path = args.get(2).unwrap_or_else(|| {
                eprintln!("Missing file path for validate");
                process::exit(1);
            });
            let raw = fs::read_to_string(path).unwrap_or_else(|e| {
                eprintln!("Failed to read {}: {}", path, e);
                process::exit(1);
            });
            match DesklandConfig::from_toml(&raw) {
                Ok(_) => println!("Configuration '{}' is valid.", path),
                Err(e) => {
                    eprintln!("Validation failed: {}", e);
                    process::exit(1);
                }
            }
        }
        _ => print_usage(),
    }
}
