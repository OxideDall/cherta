use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub thickness: f32,
    pub color: [f32; 3],
    pub opacity: f32,
    pub ttl: f32,
    pub fade_start: f32,
    pub smooth_lines: bool,
    pub min_point_distance: f32,
    pub line_feather: f32,
    pub scroll_cooldown: u64,
    pub polling_interval: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            thickness: 3.0,
            color: [1.0, 0.0, 0.0],
            opacity: 0.9,
            ttl: 2.0,
            fade_start: 1.5,
            smooth_lines: true,
            min_point_distance: 2.0,
            line_feather: 0.0,
            scroll_cooldown: 500,
            polling_interval: 50,
        }
    }
}

impl Config {
    fn path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home).join(".config/cherta/default.toml")
    }

    pub fn load() -> Self {
        let path = Self::path();
        if path.exists() {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(config) = toml::from_str(&data) {
                    eprintln!("[config] loaded from {}", path.display());
                    return config;
                }
            }
        }
        let default = Self::default();
        if let Err(e) = default.save() {
            eprintln!("[config] failed to create default: {}", e);
        }
        default
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(&path, data)?;
        eprintln!("[config] saved to {}", path.display());
        Ok(())
    }
}
