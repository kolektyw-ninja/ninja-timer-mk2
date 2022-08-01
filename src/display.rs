use std::time::{Duration, Instant};
use std::sync::mpsc::{self, TryRecvError};

use sdl2::ttf::{self, Font};
use sdl2::image::{self, InitFlag, LoadTexture};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Rect, Point};
use sdl2::render::{Canvas, TextureQuery};
use sdl2::video::Window;
use sdl2::rwops::RWops;
use sdl2::mixer::{self, Music};

use crate::state::OutputEvent;
use crate::timer::{Timer, TimerState};
use crate::assets;

pub struct Display {
    receiver: mpsc::Receiver<OutputEvent>,
    timers: Vec<Timer>,
}

const TARGET_FRAME_DURATION: Duration = Duration::from_millis(1000 / 30);

impl Display {
    pub fn new(receiver: mpsc::Receiver<OutputEvent>) -> Self {
        Self {
            receiver,
            timers: vec![Timer::new(0)],
        }
    }

    pub fn show_windows(&mut self) -> Result<(), String> {
        let sdl_context = sdl2::init()?;

        let ttf_context = ttf::init().map_err(|e| e.to_string())?;
        let _image_context = image::init(InitFlag::PNG)?;

        let _audio = sdl_context.audio()?;
        mixer::open_audio(44_100, mixer::AUDIO_S16LSB, mixer::DEFAULT_CHANNELS, 1_024)?;
        let _mixer_context = mixer::init(mixer::InitFlag::MP3);
        mixer::allocate_channels(4);
        let beep1 = Music::from_static_bytes(assets::BEEP1)?;
        let beep2 = Music::from_static_bytes(assets::BEEP2)?;
        let mut buzzer = Clip::new(Music::from_static_bytes(assets::BUZZER)?);

        let chunk_decoders_num = mixer::get_chunk_decoders_number();
        for i in 0..chunk_decoders_num {
            println!("{}", mixer::get_chunk_decoder(i));
        }

        let font = ttf_context.load_font_from_rwops(RWops::from_bytes(assets::FONT).unwrap(), 64)?;

        let video_subsystem = sdl_context.video()?;
        let displays = video_subsystem.num_video_displays()?;
        for i in 0..displays {
            let bounds = video_subsystem.display_bounds(i)?;
            println!("Display {}: {:?}", i, bounds);
        }

        let window = video_subsystem
            .window("ninja-timer", 800, 600)
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;

        let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        let texture_creator = canvas.texture_creator();
        let background = texture_creator.load_texture_bytes(assets::BACKGROUND)?;

        canvas.copy(&background, None, None)?;
        canvas.present();
        let mut event_pump = sdl_context.event_pump()?;

        let mut frame_duration = Duration::ZERO;

        let mut last_millis = 0;
        let mut last_state = self.timers[0].get_state();

        'running: loop {
            let frame_start = Instant::now();

            // Processing
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    _ => {}
                }
            }

            self.handle_messages()?;

            // Drawing
            let timer = self.timers[0];

            canvas.copy(&background, None, None)?;
            render_text(&timer.format(), &Point::new(400, 300), &font, &mut canvas)?;
            render_text(&format!("{}", frame_duration.as_millis()), &Point::new(400, 500), &font, &mut canvas)?;
            canvas.present();

            // Audio
            let millis = timer.as_millis();

            let state = timer.get_state();
            if state == TimerState::CountingDown {
                if millis / 1000 != last_millis / 1000 {
                    beep1.play(1)?;
                }
            } else if last_millis < 0 && millis >= 0 {
                beep2.play(1)?;
            } else if last_state == TimerState::Running && state == TimerState::Stopped {
                buzzer.play_duration(Duration::from_secs(2))?;
            }

            buzzer.update();

            last_millis = millis;
            last_state = state;

            // Frame padding
            frame_duration = frame_start.elapsed();
            if frame_duration < TARGET_FRAME_DURATION {
                ::std::thread::sleep(TARGET_FRAME_DURATION - frame_duration);
            }
        }

        Ok(())
    }

    fn handle_messages(&mut self) -> Result<(), String> {
        loop {
            match self.receiver.try_recv() {
                #[allow(clippy::single_match)]
                Ok(event) => match event {
                    OutputEvent::SyncTimers(timers) => {
                        self.timers = timers;
                    },
                    #[allow(unreachable_patterns)]
                    _ => (),
                },
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return Err(String::from("Receiver disconnected")),
            }
        }

        Ok(())
    }
}

struct Clip<'a> {
    music: Music<'a>,
    stop_at: Option<Instant>,
}

impl<'a> Clip<'a> {
    pub fn new(music: Music<'a>) -> Self {
        Clip {
            music,
            stop_at: None,
        }
    }

    pub fn play_duration(&mut self, duration: Duration) -> Result<(), String> {
        self.stop_at = Some(Instant::now() + duration);
        self.music.play(1)
    }

    pub fn update(&mut self) {
        if let Some(instant) = self.stop_at {
            if Instant::now() >= instant {
                Music::halt();
                self.stop_at = None;
            }
        }
    }
}

fn render_text(text: &str, point: &Point, font: &Font, canvas: &mut Canvas<Window>) -> Result<(), String> {
    let texture_creator = canvas.texture_creator();

    let surface = font
        .render(text)
        .blended(Color::RGBA(255, 255, 255, 255))
        .map_err(|e| e.to_string())?;

    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())?;

    let TextureQuery { width, height, .. } = texture.query();
    let rect = Rect::new(point.x() - (width as i32) / 2, point.y() - (height as i32) / 2, width, height);

    canvas.copy(&texture, None, rect)
}

