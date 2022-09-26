use std::env;
use std::path::{Path, PathBuf};

pub const FONT: &[u8] = include_bytes!("../static/Inconsolata-Medium.ttf");
pub const BACKGROUND: &[u8] = include_bytes!("../static/bg.png");
pub const BEEP1: &[u8] = include_bytes!("../static/beep1.wav");
pub const BEEP2: &[u8] = include_bytes!("../static/beep2.wav");
pub const BUZZER: &[u8] = include_bytes!("../static/buzzer.mp3");

pub fn get_background_path() -> PathBuf {
    let home = env::var("HOME").unwrap();
    let home_path = Path::new(&home);
    home_path.join(".config/ninja-timer/bg.png")
}
