use std::sync::mpsc;
use std::time::{Instant, Duration};

use actix_web::cookie::time::Time;

use crate::timer::{Timer, TimerState};
use crate::settings::Settings;
use crate::info::Info;

#[derive(Debug)]
pub enum InputEvent {
    StartTimers,
    StopTimer(usize),
    StopTimers,
    ResetTimers,
    RequestSync,
    SetButtonState(u8, bool),
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
    SetDisplay(bool),
}

pub struct StateManager {
    listeners: Vec<mpsc::Sender<OutputEvent>>,
    timers: Vec<Timer>,
    settings: Settings,
    info: Info,
    reset_at: Instant,
    started_at: Instant,
    toggled_debug_at: Instant,
    display_visible: bool,
}

impl StateManager {

    pub fn new() -> Self {
        let settings = Settings::load().unwrap_or_else(|_| {
            let default = Settings::default();
            default.save().unwrap();
            default
        });

        let info = Info::get().unwrap();

        Self {
            listeners: vec![],
            timers: (0..info.number_displays).map(|_| Timer::new(settings.countdown)).collect(),
            settings,
            info,
            reset_at: Instant::now(),
            started_at: Instant::now(),
            toggled_debug_at: Instant::now(),
            display_visible: true,
        }
    }

    pub fn add_listener(&mut self, listener: mpsc::Sender<OutputEvent>) {
        self.listeners.push(listener);
    }

    pub fn process(&mut self, event: InputEvent) -> Result<(), String> {
        match event {
            InputEvent::StartTimers => {
                let is_reset = self.get_timer_mut(0)?.get_state() == TimerState::Reset;

                if is_reset && Instant::now() - self.reset_at > Duration::from_secs(1) {
                    for (i, timer) in self.timers.iter_mut().enumerate() {
                        if let Err(msg) = timer.start() {
                            eprintln!("Timer {} couldn't be started: {}", i, msg);
                        }
                        self.started_at = Instant::now();
                    }

                    self.notify_listeners(&OutputEvent::SyncTimers(self.timers.clone()))?;
                } else if Instant::now() - self.started_at > Duration::from_secs(1) {
                    for timer in &mut self.timers {
                        timer.reset();
                        self.reset_at = Instant::now();
                    }
                    
                    self.reset_at = Instant::now();
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
            InputEvent::StopTimers => {
                for timer in self.timers.iter_mut() {
                    timer.stop()?;
                }
                self.notify_listeners(&OutputEvent::SyncTimers(self.timers.clone()))?;
            },
            InputEvent::ResetTimers => {
                for timer in &mut self.timers {
                    timer.reset();
                }
                
                self.notify_listeners(&OutputEvent::SyncTimers(self.timers.clone()))?;
            },
            InputEvent::RequestSync => {
                self.notify_listeners(&OutputEvent::SyncTimers(self.timers.clone()))?;
                self.notify_listeners(&OutputEvent::SyncSettings(self.settings.clone()))?;
                self.notify_listeners(&OutputEvent::SyncInfo(self.info.clone()))?;
            },
            InputEvent::SetButtonState(button_id, pressed) => {
                if pressed && (button_id == 1 || button_id == 2) {
                    if self.get_timer_mut((button_id - 1).into())?.stop().is_ok() {
                        self.notify_listeners(&OutputEvent::SyncTimers(self.timers.clone()))?;
                    }
                }

                if button_id == 9 && Instant::now() - self.toggled_debug_at > Duration::from_secs_f32(0.5) {
                    self.settings.show_debug = !self.settings.show_debug;
                    self.notify_listeners(&OutputEvent::SyncSettings(self.settings.clone()))?;
                    self.settings.save().unwrap();
                    self.toggled_debug_at = Instant::now();
                }
            },
            InputEvent::SetDebug(debug) => {
                self.settings.show_debug = debug;
                self.notify_listeners(&OutputEvent::SyncSettings(self.settings.clone()))?;
                self.settings.save().unwrap();
            },
            InputEvent::SetCountdown(countdown) => {
                self.settings.countdown = countdown;
                self.timers = (0..self.info.number_displays).map(|_| Timer::new(self.settings.countdown)).collect();
                self.notify_listeners(&OutputEvent::SyncSettings(self.settings.clone()))?;
                self.notify_listeners(&OutputEvent::SyncTimers(self.timers.clone()))?;

                self.settings.save().unwrap();
            },
            InputEvent::ReloadBackground => {
                self.notify_listeners(&OutputEvent::ReloadBackground)?;
            },
            InputEvent::ToggleDisplay => {
                self.display_visible = !self.display_visible;
                self.notify_listeners(&OutputEvent::SetDisplay(self.display_visible))?;
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
