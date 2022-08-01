use std::time::{Instant, Duration};

#[derive(Debug, Clone, Copy)]
pub struct Timer {
    started_at: Option<Instant>,
    stopped_at: Option<Instant>,
    countdown_duration: Duration,
}

#[derive(Eq, PartialEq)]
pub enum TimerState {
    Reset,
    CountingDown,
    Running,
    Stopped,
}

impl Timer {
    pub fn new(countdown_seconds: u64) -> Self {
        Self {
            started_at: None,
            stopped_at: None,
            countdown_duration: Duration::from_secs(countdown_seconds),
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        if self.get_state() == TimerState::Reset {
            self.started_at = Some(Instant::now());
            Ok(())
        } else {
            Err("Timer hasn't been reset".to_string())
        }
    }

    pub fn stop(&mut self) -> Result<(), String> {
        if self.get_state() == TimerState::Running {
            self.stopped_at = Some(Instant::now());
            Ok(())
        } else {
            Err("Timer isn't running".to_string())
        }
    }

    pub fn as_millis(&self) -> i128 {
        match self.stopped_at {
            Some(stop) => self.started_at.map_or(0, |start| (stop - start).as_millis() as i128 - self.countdown_duration.as_millis() as i128),
            None => self.started_at.map_or(0, |start| (Instant::now() - start).as_millis() as i128 - self.countdown_duration.as_millis() as i128),
        }
    }

    pub fn reset(&mut self) {
        self.started_at = None;
        self.stopped_at = None;
    }

    pub fn get_state(&self) -> TimerState {
        match self.started_at {
            None => TimerState::Reset,
            Some(_) => {
                if self.as_millis() < 0 {
                    TimerState::CountingDown
                } else {
                    match self.stopped_at {
                        Some(_) => TimerState::Stopped,
                        None => TimerState::Running,
                    }
                }
            },
        }
    }

    pub fn format(&self) -> String {
        let elapsed_millis = self.as_millis().abs();

        match self.get_state() {
            TimerState::CountingDown => {
                let mut seconds_left = elapsed_millis / 1000;

                if elapsed_millis % 1000 > 0 {
                    seconds_left += 1;
                }

                format!("{}", seconds_left)
            },
            _ => {
                let hundredths = (elapsed_millis / 10) % 100;
                let seconds = (elapsed_millis / 1000) % 60;
                let minutes = elapsed_millis / (1000 * 60);

                format!("{:02}:{:02}.{:02}", minutes, seconds, hundredths)
            }
        }
    }
}
