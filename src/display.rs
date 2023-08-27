use std::borrow::BorrowMut;
use std::time::{Duration, Instant};
use std::sync::mpsc::{self, TryRecvError};

use sdl2::ttf::{self, Font, Sdl2TtfContext};
use sdl2::image::{self, InitFlag, LoadTexture};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Rect, Point};
use sdl2::render::{Canvas, TextureQuery, Texture, TextureCreator};
use sdl2::video::{Window, FullscreenType, WindowContext};
use sdl2::rwops::RWops;
use sdl2::mixer::{self, Music};

use crate::state::OutputEvent;
use crate::timer::{Timer, TimerState};
use crate::assets::{self, get_background_path};
use crate::settings::Settings;
use crate::info::Info;

pub struct Display {
    receiver: mpsc::Receiver<OutputEvent>,
    timers: Vec<Timer>,
    settings: Option<Settings>,
    info: Option<Info>,
    should_reload_background: bool,
    is_visible: bool,
}

const TARGET_FRAME_DURATION: Duration = Duration::from_millis(1000 / 30);

struct WindowData {
    canvas: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,
    last_millis: i128,
    last_state: TimerState,
}

impl Display {
    pub fn new(receiver: mpsc::Receiver<OutputEvent>) -> Self {
        Self {
            receiver,
            timers: vec![],
            settings: None,
            info: None,
            should_reload_background: true,
            is_visible: true,
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

        let video_subsystem = sdl_context.video()?;
        let displays = video_subsystem.num_video_displays()?;
        let display_bounds: Vec<_> = (0..displays).map(|i| { video_subsystem.display_bounds(i).unwrap()}).collect();

        for (i, bounds) in display_bounds.iter().enumerate() {
            eprintln!("Display {}: {:?}", i, bounds);
        }

        // let width = display_bounds[0].w as u32;
        // let height = display_bounds[0].h as u32;
        let width = 800;
        let height = 600;

        sdl_context.mouse().show_cursor(false);

        let mut font = ResizeableFont::load_from_bytes(&ttf_context, assets::FONT, 64)?;
        let debug_font = ResizeableFont::load_from_bytes(&ttf_context, assets::FONT, 20)?;

        let mut builder = video_subsystem.window("ninja-timer", width, height);

        builder.opengl();
        
        let mut windows = vec![];
        
        for i in 0..2 {
            let window = builder.build().map_err(|e| e.to_string())?;

            let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
            let texture_creator = canvas.texture_creator();

            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas.clear();
            canvas.present();

            windows.push(WindowData{ 
                canvas, 
                texture_creator,
                last_millis: 0,
                last_state: TimerState::Reset,
            });
        }

        let bg_path = get_background_path();

        let mut event_pump = sdl_context.event_pump()?;

        let mut frame_duration: Duration = TARGET_FRAME_DURATION;
        let mut start_sound_played = false;

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

            let mut backgrounds = vec![None, None];

            for (i, window) in windows.iter_mut().enumerate() {
                let window_enabled = i < self.timers.len();
                self.sync_fullscreen(window.canvas.window_mut());

                if self.is_visible && window_enabled {
                    window.canvas.window_mut().show();
                } else {
                    window.canvas.window_mut().hide();
                }

                // if self.should_toggle_visibility {
                //     if self.is_shown {
                //         println!("Hiding display");
                //         window.canvas.window_mut().hide();
                //     } else {
                //         println!("Showing display");
                //         window.canvas.window_mut().show();
                //     }

                //     self.is_shown = !self.is_shown;
                //     self.should_toggle_visibility = false;
                // }

                if self.should_reload_background {
                    backgrounds[i] = if bg_path.is_file() {
                        window.texture_creator.load_texture(&bg_path).ok()
                    } else {
                        None
                    };

                    if i == self.timers.len() - 1 {
                        self.should_reload_background = false;
                    }
                }

                if i > self.timers.len() - 1 {
                    continue;
                }

                // Drawing
                let viewport = window.canvas.viewport();
                let width = viewport.width();
                let height = viewport.height();

                let timer = self.timers[i];
                font.set_size(height as u16 / 3);

                window.canvas.set_draw_color(Color::RGB(0, 0, 0));
                window.canvas.clear();

                if let Some(ref bg) = backgrounds[i] {
                    window.canvas.copy(bg, None, None)?;
                }

                let color = match timer.get_state() {
                    TimerState::CountingDown => Color::RGB(255, 0, 0),
                    TimerState::Stopped => Color::RGB(0, 255, 0),
                    _ => Color::RGB(255, 255, 255),
                };

                render_text(
                    &timer.format(),
                    &Point::new(width as i32 / 2, height as i32 / 2),
                    &font.inner,
                    &mut window.canvas,
                    &color,
                    Align::Center,
                )?;

                // Debug
                if self.debug_enabled() {
                    let ips = if let Some(ref info) = self.info {
                        info.ips.clone()
                    } else {
                        vec![]
                    };

                    window.canvas.set_draw_color(Color::RGB(255, 255, 255));
                    window.canvas.fill_rect(Rect::new(0, 0, width, 200))?;

                    render_text(
                        &format!("IP: {}", ips.join(", ")),
                        &Point::new(10, 10),
                        &debug_font.inner,
                        &mut window.canvas,
                        &Color::RGB(0, 0, 0),
                        Align::TopLeft,
                    )?;

                    render_text(
                        &format!("FPS: {:.02}", 1000/frame_duration.as_millis()),
                        &Point::new(10, 30),
                        &debug_font.inner,
                        &mut window.canvas,
                        &Color::RGB(0, 0, 0),
                        Align::TopLeft,
                    )?;

                }

                window.canvas.present();

                // Audio
                let millis = timer.as_millis();

                let state = timer.get_state();
                if state == TimerState::CountingDown {
                    if i == 0 && millis / 1000 != window.last_millis / 1000 {
                        beep1.play(1)?;
                    }
                } else if (window.last_millis < 0 && millis >= 0) || (state == TimerState::Running && !start_sound_played) {
                    if i == 0 {
                        beep2.play(1)?;
                        start_sound_played = true;
                    }
                } else if window.last_state == TimerState::Running && state == TimerState::Stopped {
                    buzzer.play_duration(Duration::from_secs(2))?;
                }

                if i == 0 && (state == TimerState::CountingDown || state == TimerState::Stopped || state == TimerState::Reset) {
                    start_sound_played = false;
                }

                window.last_millis = millis;
                window.last_state = state;
            }

            buzzer.update();

            // Frame padding
            frame_duration = frame_start.elapsed();
            if frame_duration < TARGET_FRAME_DURATION {
                ::std::thread::sleep(TARGET_FRAME_DURATION - frame_duration);
            }
        }

        Ok(())
    }

    fn debug_enabled(&self) -> bool {
        if let None = self.settings {
            return false
        }

        if let Some(Settings { show_debug: false, .. }) = self.settings {
            return false
        }

        true
    }

    fn sync_fullscreen(&self, window: &mut Window) {
        match self.settings {
            None => return,
            Some(settings) => {
                let fullscreen = window.fullscreen_state();
                if fullscreen == FullscreenType::Off && settings.fullscreen {
                    window.set_fullscreen(FullscreenType::True).unwrap();
                } else if fullscreen == FullscreenType::True && !settings.fullscreen {
                    window.set_fullscreen(FullscreenType::Off).unwrap();
                }
            }
        }
    }

    fn handle_messages(&mut self) -> Result<(), String> {
        loop {
            match self.receiver.try_recv() {
                #[allow(clippy::single_match)]
                Ok(event) => match event {
                    OutputEvent::SyncTimers(timers) => {
                        self.timers = timers;
                    },
                    OutputEvent::SyncSettings(settings) => self.settings = Some(settings),
                    OutputEvent::SyncInfo(info) => self.info = Some(info),
                    OutputEvent::ReloadBackground => self.should_reload_background = true,
                    OutputEvent::SetDisplay(x) => self.is_visible = x,
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

struct ResizeableFont<'ttf_module, 'rwops> {
    ctx: &'ttf_module Sdl2TtfContext,
    bytes: &'static [u8],
    inner: Font<'ttf_module, 'rwops>,
    size: u16,
}

impl<'a, 'b> ResizeableFont<'a, 'b> {
    pub fn load_from_bytes(ctx: &'a Sdl2TtfContext, bytes: &'static [u8], size: u16) -> Result<Self, String> {
        let font = Self {
            ctx,
            bytes,
            inner: ctx.load_font_from_rwops(RWops::from_bytes(bytes)?, size)?,
            size,
        };

        Ok(font)
    }

    pub fn set_size(&mut self, size: u16) {
        if size != self.size {
            self.inner = self.ctx.load_font_from_rwops(RWops::from_bytes(self.bytes).unwrap(), size).unwrap();
            self.size = size;
        }
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

enum Align {
    Center,
    TopLeft,
}

fn render_text(text: &str, point: &Point, font: &Font, canvas: &mut Canvas<Window>, color: &Color, align: Align) -> Result<(), String> {
    let texture_creator = canvas.texture_creator();

    let surface = font
        .render(text)
        .blended(*color)
        .map_err(|e| e.to_string())?;

    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())?;

    let TextureQuery { width, height, .. } = texture.query();
    let rect = match align {
        Align::Center =>
            Rect::new(point.x() - (width as i32) / 2, point.y() - (height as i32) / 2, width, height),
        Align::TopLeft => Rect::new(point.x(), point.y(), width, height),
    };

    canvas.copy(&texture, None, rect)
}

