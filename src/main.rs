extern crate sdl2;

mod display;
mod timer;
mod state;
mod assets;

use std::thread;
use std::sync::mpsc;
use std::time::Duration;

use display::Display;
use timer::Timer;
use state::{StateManager, OutputEvent, InputEvent};


pub fn main() -> Result<(), String> {
    let (tx, rx) = mpsc::channel::<OutputEvent>();

    let mut display = Display::new(rx);
    let handle = thread::spawn(move || {
        display.show_windows().expect("Could not init windows");
    });

    let mut state_manager = StateManager::new();

    state_manager.add_listener(tx);
    state_manager.process(InputEvent::StartTimer(0))?;

    handle.join().unwrap();
    Ok(())
}

