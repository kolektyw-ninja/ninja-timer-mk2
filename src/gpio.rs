use std::sync::mpsc::Sender;
use std::thread::{spawn, JoinHandle};

use rppal::gpio::{Gpio, Trigger, Level};

use crate::state::InputEvent;

const PIN_NUMBER: u8 = 17;

pub fn spawn_gpio(sender: Sender<InputEvent>) -> JoinHandle<()> {
    spawn(move || {
        //if !cfg!(target_arch = "arm") {
            //println!("arch != 'arm', skipping GPIO");
            //return
        //}

        let gpio = Gpio::new().unwrap();
        let mut pin = gpio.get(PIN_NUMBER).unwrap().into_input_pulldown();
        pin.set_interrupt(Trigger::Both).unwrap();

        loop {
            let poll = pin.poll_interrupt(false, None);
            if let Ok(Some(level)) = poll {
                let button = level == Level::Low;
                sender.send(InputEvent::SetButtonState(button)).unwrap();
            }
        }
    })
}
