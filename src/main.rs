mod display;
mod timer;
mod state;
mod assets;
mod web;
mod broadcast;

use std::thread;
use std::sync::mpsc;
use std::time::Duration;

use display::Display;
use timer::Timer;
use state::{StateManager, OutputEvent, InputEvent};
use web::spawn_server;


pub fn main() -> Result<(), String> {
    let (input_tx, input_rx) = mpsc::channel();
    let _server_handle = spawn_server(input_tx);

    let (display_tx, display_rx) = mpsc::channel();
    let mut display = Display::new(display_rx);
    let display_handle = thread::spawn(move || {
        display.show_windows().unwrap();
    });

    let mut state_manager = StateManager::new();
    state_manager.add_listener(display_tx);

    for event in input_rx {
        state_manager.process(event).unwrap();
    }

    display_handle.join().unwrap();

    Ok(())
}

