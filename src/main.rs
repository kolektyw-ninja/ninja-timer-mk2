mod display;
mod timer;
mod state;
mod assets;
mod web;
mod broadcast;
mod gpio;
mod info;
mod settings;

use std::thread;
use std::sync::mpsc;

use display::Display;
use state::{StateManager};
use web::spawn_server;
use gpio::spawn_gpio;

pub fn main() -> Result<(), String> {
    wait_for_network();

    let (input_tx, input_rx) = mpsc::channel();
    let (output_tx, output_rx) = mpsc::channel();
    let _server_handle = spawn_server(input_tx.clone(), output_rx);

    let (display_tx, display_rx) = mpsc::channel();
    let mut display = Display::new(display_rx);
    let display_handle = thread::spawn(move || {
        display.show_windows().unwrap();
    });

    let _gpio_handle = spawn_gpio(input_tx);

    let mut state_manager = StateManager::new();
    state_manager.add_listener(display_tx);
    state_manager.add_listener(output_tx);

    for event in input_rx {
        state_manager.process(event).unwrap();
    }

    display_handle.join().unwrap();

    Ok(())
}

fn wait_for_network() {
    let mut counter = 0;

    loop {
        let ips = info::get_ips().unwrap();
        if ips.len() > 0 {
            break
        }

        println!("Waiting for network #{}", counter);
        std::thread::sleep(std::time::Duration::from_secs(1));
        counter += 1;
    }
}
