use std::path::PathBuf;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub global: Global,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Global {
    pub last_width: f32,
    pub last_height: f32,
    pub last_pos_x: f32,
    pub last_pos_y: f32,
}

impl Config {
    pub fn load_or_create() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config = toml::from_str(&content)?;
            return Ok(config);
        }

        // Create default config
        let default_config = Config {
            global: Global {
                last_width: 1063.0,
                last_height: 660.0,
                last_pos_x: 100.0,
                last_pos_y: 100.0,
            },
        };

        default_config.save()?;
        Ok(default_config)
    }

    pub fn config_path() -> Result<PathBuf> {
        let config_dir = Self::config_dir()?;
        std::fs::create_dir_all(&config_dir)?;
        Ok(config_dir.join(crate::MAIN_CONFIG_FILE_NAME))
    }

    pub fn config_dir() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Could not determine config directory"))?
            .join(crate::APP_PATH);
        Ok(config_dir)
    }

    pub fn profile_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join(crate::PROFILE_FILE_NAME))
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        let content = toml::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;
        Ok(())
    }

    // Check if size or position of main window was changed
    pub fn update_window_state(&mut self, width: f32, height: f32, x: f32, y: f32) {
        if self.global.last_width != width || self.global.last_height != height || self.global.last_pos_x != x || self.global.last_pos_y != y {
            self.global.last_width = width;
            self.global.last_height = height;
            self.global.last_pos_x = x;
            self.global.last_pos_y = y;
        }
    }
}
