use std::io::Result;
use std::fs;
use std::env;
use std::path::{Path, PathBuf};

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub countdown: u64,
    pub show_debug: bool,
    pub fullscreen: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            countdown: 3,
            show_debug: false,
            fullscreen: true,
        }
    }
}

impl Settings {
    // load from file
    pub fn load() -> Result<Self> {
        let settings_string = fs::read_to_string(Self::get_path())?;
        Ok(serde_json::from_str(&settings_string).unwrap())
    }

    pub fn save(&self) -> Result<()> {
        let json = serde_json::to_string(self).unwrap();
        let path = Self::get_path();
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, json)
    }

    fn get_path() -> PathBuf {
        let home = env::var("HOME").unwrap();
        let home_path = Path::new(&home);
        home_path.join(".config/ninja-timer/settings.json")
    }
}
