use std::process::Command;
use std::io::Result;

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
