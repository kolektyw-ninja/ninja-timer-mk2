use std::sync::mpsc::Sender;
use std::thread::{spawn, JoinHandle};
use std::time::{Duration, Instant};
use std::collections::HashMap;

use rppal::gpio::{Gpio, Trigger, Level};

use crate::state::InputEvent;

const BUZZER_PIN: u8 = 17;
const DEBUG_PIN: u8 = 27;
const DISPLAY_PIN: u8 = 22;
const POWER_PIN: u8 = 23;

const DEBOUNCE_DURATION: Duration = Duration::from_millis(50);

struct Button {
    real_level: Option<Level>,
    last_change: Option<Instant>,
    last_returned: Option<Level>,
    debounce_duration: Option<Duration>,
}

impl Button {
    fn new(debounce_duration: Option<Duration>) -> Self {
        Button {
            real_level: None,
            last_change: None,
            last_returned: None,
            debounce_duration,
        }
    }

    fn update(&mut self, level: Level) {
        self.real_level = Some(level);
        self.last_change = Some(Instant::now());
    }

    fn get_changed_level(&mut self) -> Option<Level> {
        if self.real_level == None || self.real_level == self.last_returned {
            return None;
        }

        match self.debounce_duration {
            None => {
                self.last_returned = self.real_level;
                self.real_level
            },
            Some(duration) => {
                let time_since_change = Instant::now() - self.last_change.unwrap();

                if time_since_change >= duration {
                    self.last_returned = self.real_level;
                    self.real_level
                } else {
                    None
                }
            }
        }
    }
}


fn spawn_gpio(sender: Sender<InputEvent>) -> JoinHandle<()> {
    spawn(move || {
        let gpio = Gpio::new().unwrap();

        let power_pin = gpio.get(POWER_PIN).unwrap().into_output_high();

        let mut buttons = HashMap::from([
            (BUZZER_PIN, Button::new(None)),
            (DEBUG_PIN, Button::new(Some(DEBOUNCE_DURATION))),
            (DISPLAY_PIN, Button::new(Some(DEBOUNCE_DURATION))),
        ]);

        let input_pins: Vec<_> = buttons.keys().into_iter().map(|pin_number| {
            let mut pin = gpio.get(*pin_number).unwrap().into_input_pulldown();
            pin.set_interrupt(Trigger::Both).unwrap();
            pin
        }).collect();

        // Get Vec<&InputPin> from Vec<InputPin>
        let poll_pins: Vec<_> = input_pins.iter().collect();

        loop {
            let poll = gpio.poll_interrupts(&poll_pins, false, Some(Duration::from_millis(10)));

            if let Ok(Some((pin, level))) = poll {
                if let Some(button) = buttons.get_mut(&pin.pin()) {
                    button.update(level);
                }
            }

            for (pin, button) in buttons.iter_mut() {
                if let Some(level) = button.get_changed_level() {
                    match *pin {
                        BUZZER_PIN => {
                            sender.send(InputEvent::SetButtonState(level == Level::Low)).unwrap();
                        },
                        DEBUG_PIN => {
                            if level == Level::High {
                                sender.send(InputEvent::ToggleDebug).unwrap();
                            }
                        },
                        DISPLAY_PIN => {
                            if level == Level::High {
                                sender.send(InputEvent::ToggleDisplay).unwrap();
                            }
                        },
                        _ => (),
                    };
                }
            }
        }
    })
}
