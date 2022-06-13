use std::time::{Duration, Instant};
use std::path::Path;
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

use crate::state::OutputEvent;
use crate::timer::Timer;
use crate::assets;

pub struct Display {
    receiver: mpsc::Receiver<OutputEvent>,
    text: String,
    timers: Vec<Timer>,
}

const TARGET_FRAME_DURATION: Duration = Duration::from_millis(1000 / 30);

impl Display {
    pub fn new(receiver: mpsc::Receiver<OutputEvent>) -> Self {
        Self {
            receiver,
            text: String::from("hello world"),
            timers: vec![Timer::new(0)],
        }
    }

    pub fn show_windows(&mut self) -> Result<(), String> {
        let sdl_context = sdl2::init()?;
        let ttf_context = ttf::init().map_err(|e| e.to_string())?;
        let _image_context = image::init(InitFlag::PNG)?;

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

        'running: loop {
            let frame_start = Instant::now();

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

            canvas.copy(&background, None, None)?;
            render_text(&self.timers[0].format(), &Point::new(400, 300), &font, &mut canvas)?;
            render_text(&format!("{}", frame_duration.as_millis()), &Point::new(400, 500), &font, &mut canvas)?;
            canvas.present();

            self.handle_messages()?;
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
                Ok(event) => match event {
                    OutputEvent::SyncTimers(timers) => {
                        self.timers = timers;
                    },
                    _ => (),
                },
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return Err(String::from("Receiver disconnected")),
            }
        }

        Ok(())
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

