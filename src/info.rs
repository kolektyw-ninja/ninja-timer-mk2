use std::process::Command;
use std::io::Result;

#[derive(Debug, Clone)]
pub struct Info {
    pub ips: Vec<String>,
    pub number_displays: usize,
}

impl Info {
    pub fn get() -> Result<Self> {
        let info = Info {
            ips: get_ips()?,
            number_displays: get_number_displays()?,
        };

        Ok(info)
    }
}

pub fn get_number_displays() -> Result<usize> {
    if cfg!(target_os = "linux") {
        let output = Command::new("xrandr").output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let displays = stdout
            .lines()
            .filter(|x| x.split_whitespace().nth(1) == Some("connected"))
            .count();

        Ok(displays)
    } else {
        Ok(1)
    }
} 

#[cfg(target_os = "linux")]
pub fn get_ips() -> Result<Vec<String>> {
    let output = Command::new("ip")
        .arg("addr")
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let ips: Vec<String> = stdout
        .lines()
        .filter(|x| x.trim().starts_with("inet "))
        .map(|x| x
            .split_whitespace()
            .nth(1).unwrap()
            .split("/")
            .nth(0).unwrap()
            .into()
        )
        .filter(|x| x != "127.0.0.1")
        .collect();

    Ok(ips)
}

#[cfg(target_os = "macos")]
pub fn get_ips() -> Result<Vec<String>> {
    let output = Command::new("ifconfig")
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let ips: Vec<String> = stdout
        .lines()
        .filter(|x| x.trim().starts_with("inet "))
        .map(|x| x
            .split_whitespace()
            .nth(1).unwrap()
            .into()
        )
        .filter(|x| x != "127.0.0.1")
        .collect();

    Ok(ips)
}
