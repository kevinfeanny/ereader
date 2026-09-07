use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const CONFIG_PATH: &str = "/home/pi/.config/ereader/config.json";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub font_size:    f32,
    pub line_height:  u32,
    pub margin_top:   u32,
    pub margin_bottom: u32,
    pub margin_left:  u32,
    pub margin_right: u32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            font_size:     22.0,
            line_height:   28,
            margin_top:    86,
            margin_bottom: 86,
            margin_left:   94,
            margin_right:  94,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = Path::new(CONFIG_PATH);
        if path.exists() {
            fs::read_to_string(path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Config::default()
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Path::new(CONFIG_PATH);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn increase_font(&mut self) {
        if self.font_size < 40.0 {
            self.font_size  += 2.0;
            self.line_height = (self.font_size as u32) + 6;
        }
    }

    pub fn decrease_font(&mut self) {
        if self.font_size > 14.0 {
            self.font_size  -= 2.0;
            self.line_height = (self.font_size as u32) + 6;
        }
    }

    pub fn max_lines(&self) -> usize {
        let usable_height = self.margin_top + self.margin_bottom;
        let text_height   = 480u32.saturating_sub(usable_height) - 25;
        (text_height / self.line_height) as usize
    }

    pub fn chars_per_line(&self) -> usize {
        let usable_width = 800u32
            .saturating_sub(self.margin_left + self.margin_right);
        (usable_width / (self.font_size as u32 / 2)) as usize
    }
}