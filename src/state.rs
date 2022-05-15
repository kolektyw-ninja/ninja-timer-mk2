use std::sync::mpsc;

use crate::timer::Timer;

#[derive(Debug)]
pub enum InputEvent {
    StartTimer(usize),
    StopTimer(usize),
    ResetTimer(usize),
}

#[derive(Debug, Clone)]
pub enum OutputEvent {
    SyncTimers(Vec<Timer>),
}

pub struct StateManager {
    listeners: Vec<mpsc::Sender<OutputEvent>>,
    timers: Vec<Timer>,
}

impl StateManager {

    pub fn new() -> Self {
        Self {
            listeners: vec![],
            timers: vec![Timer::new(3)],
        }
    }

    pub fn add_listener(&mut self, listener: mpsc::Sender<OutputEvent>) {
        self.listeners.push(listener);
    }

    pub fn process(&mut self, event: InputEvent) -> Result<(), String> {
        match event {
            InputEvent::StartTimer(i) => {
                match self.timers.get_mut(i) {
                    Some(timer) => {
                        timer.start();

                        for listener in &self.listeners {
                            listener.send(OutputEvent::SyncTimers(self.timers.clone())).map_err(|e| e.to_string())?;
                        }
                    },
                    None => return Err(format!("Timer index {} is out of bounds", i)),
                }
            },
            _ => return Err(format!("Couldn't process: {:?}", event)),
        }

        Ok(())
    }

    fn notify_listeners(&mut self, event: &OutputEvent) -> Result<(), String> {
        let errors: Vec<_> = self.listeners
            .iter()
            .map(|listener| listener.send(event.clone()))
            .filter(Result::is_err)
            .collect();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(String::from("One or more listeners failed"))
        }
    }
}