extern crate sdl2;

mod display;

use std::thread;
use std::sync::mpsc;
use std::time::Duration;

use display::Display;

pub fn main() -> Result<(), String> {
    let (tx, rx) = mpsc::channel();

    let mut display = Display::new(rx);
    let handle = thread::spawn(move || {
        display.show_windows().expect("Could not init windows");
    });

    tx.send(String::from("5")).unwrap();
    thread::sleep(Duration::new(0, 1_000_000_000u32));
    tx.send(String::from("4")).unwrap();
    thread::sleep(Duration::new(0, 1_000_000_000u32));
    tx.send(String::from("3")).unwrap();
    thread::sleep(Duration::new(0, 1_000_000_000u32));
    tx.send(String::from("2")).unwrap();
    thread::sleep(Duration::new(0, 1_000_000_000u32));
    tx.send(String::from("1")).unwrap();
    thread::sleep(Duration::new(0, 1_000_000_000u32));
    tx.send(String::from("0")).unwrap();
    thread::sleep(Duration::new(0, 1_000_000_000u32));

    handle.join().unwrap();
    Ok(())
}

