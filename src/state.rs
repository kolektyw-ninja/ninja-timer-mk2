use std::sync::mpsc;

use crate::timer::Timer;
use crate::settings::Settings;
use crate::info::Info;

#[derive(Debug)]
pub enum InputEvent {
    StartTimer(usize),
    StopTimer(usize),
    ResetTimer(usize),
    RequestSync,
    SetButtonState(bool),
    SetDebug(bool),
    SetCountdown(u64),
    ReloadBackground,
    ToggleDisplay,
    ToggleDebug,
}

#[derive(Debug, Clone)]
pub enum OutputEvent {
    SyncTimers(Vec<Timer>),
    SyncSettings(Settings),
    SyncInfo(Info),
    ReloadBackground,
    ToggleDisplay,
}

pub struct StateManager {
    listeners: Vec<mpsc::Sender<OutputEvent>>,
    timers: Vec<Timer>,
    settings: Settings,
    info: Info,
}

impl StateManager {

    pub fn new() -> Self {
        let settings = Settings::load().unwrap_or_else(|_| {
            let default = Settings::default();
            default.save().unwrap();
            default
        });

        Self {
            listeners: vec![],
            timers: vec![Timer::new(settings.countdown)],
            settings,
            info: Info::get().unwrap(),
        }
    }

    pub fn add_listener(&mut self, listener: mpsc::Sender<OutputEvent>) {
        self.listeners.push(listener);
    }

    pub fn process(&mut self, event: InputEvent) -> Result<(), String> {
        match event {
            InputEvent::StartTimer(i) => {
                if let Err(msg) = self.get_timer_mut(i)?.start() {
                    eprintln!("Timer {} couldn't be started: {}", i, msg);
                } else {
                    self.notify_listeners(&OutputEvent::SyncTimers(self.timers.clone()))?;
                }
            },
            InputEvent::StopTimer(i) => {
                if let Err(msg) = self.get_timer_mut(i)?.stop() {
                    eprintln!("Timer {} couldn't be stopped: {}", i, msg);
                } else {
                    self.notify_listeners(&OutputEvent::SyncTimers(self.timers.clone()))?;
                }
            },
            InputEvent::ResetTimer(i) => {
                self.get_timer_mut(i)?.reset();
                self.notify_listeners(&OutputEvent::SyncTimers(self.timers.clone()))?;
            },
            InputEvent::RequestSync => {
                self.notify_listeners(&OutputEvent::SyncTimers(self.timers.clone()))?;
                self.notify_listeners(&OutputEvent::SyncSettings(self.settings.clone()))?;
                self.notify_listeners(&OutputEvent::SyncInfo(self.info.clone()))?;
            },
            InputEvent::SetButtonState(state) => {
                if state {
                    if let Ok(_) = self.get_timer_mut(0)?.stop() {
                        self.notify_listeners(&OutputEvent::SyncTimers(self.timers.clone()))?;
                    }
                }
            },
            InputEvent::SetDebug(debug) => {
                self.settings.show_debug = debug;
                self.notify_listeners(&OutputEvent::SyncSettings(self.settings.clone()))?;
                self.settings.save().unwrap();
            },
            InputEvent::SetCountdown(countdown) => {
                self.settings.countdown = countdown;
                self.timers = vec![Timer::new(countdown)];
                self.notify_listeners(&OutputEvent::SyncSettings(self.settings.clone()))?;
                self.notify_listeners(&OutputEvent::SyncTimers(self.timers.clone()))?;

                self.settings.save().unwrap();
            },
            InputEvent::ReloadBackground => {
                self.notify_listeners(&OutputEvent::ReloadBackground)?;
            },
            InputEvent::ToggleDisplay => {
                self.notify_listeners(&OutputEvent::ToggleDisplay)?;
            },
            InputEvent::ToggleDebug => {
                self.settings.show_debug = !self.settings.show_debug;
                self.notify_listeners(&OutputEvent::SyncSettings(self.settings.clone()))?;
                self.settings.save().unwrap();
            },
            #[allow(unreachable_patterns)]
            _ => return Err(format!("Couldn't process: {:?}", event)),
        }

        Ok(())
    }

    pub fn sync_all(&mut self) -> Result<(), String> {
        self.notify_listeners(&OutputEvent::SyncTimers(self.timers.clone()))?;
        self.notify_listeners(&OutputEvent::SyncSettings(self.settings.clone()))?;
        self.notify_listeners(&OutputEvent::SyncInfo(self.info.clone()))?;

        Ok(())
    }

    fn get_timer_mut(&mut self, id: usize) -> Result<&mut Timer, String> {
        match self.timers.get_mut(id) {
            Some(timer) => Ok(timer),
            None => Err(format!("Timer index {} is out of bounds", id)),
        }
    }

    fn notify_listeners(&mut self, event: &OutputEvent) -> Result<(), String> {
        let errors = self.listeners
            .iter()
            .map(|listener| listener.send(event.clone()))
            .filter(Result::is_err);

        for error in errors {
            // TODO: remove instead
            println!("{:?}", error)
        }

        Ok(())
    }
}
