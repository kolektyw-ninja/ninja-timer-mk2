extern crate sdl2;

mod display;

use std::thread;

use display::show_windows;

pub fn main() -> Result<(), String> {

    let handle = thread::spawn(|| {
        show_windows().expect("Could not init windows");
    });

    handle.join().unwrap();
    Ok(())
}

