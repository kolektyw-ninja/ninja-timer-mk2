extern crate sdl2;

use std::time::Duration;
use std::path::Path;

use sdl2::ttf;
use sdl2::ttf::Font;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Rect, Point};
use sdl2::render::{Canvas, TextureQuery};
use sdl2::video::Window;

pub fn show_windows() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let ttf_context = ttf::init().map_err(|e| e.to_string())?;
    let video_subsystem = sdl_context.video()?;
    let font = ttf_context.load_font(Path::new("./static/Inconsolata-Medium.ttf"), 64)?;

    let window = video_subsystem
        .window("ninja-timer", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
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

        canvas.clear();
        render_text("Hello world", &Point::new(400, 300), &font, &mut canvas)?;
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        // The rest of the game loop goes here...
    }

    Ok(())
}

fn render_text(text: &'static str, point: &Point, font: &Font, canvas: &mut Canvas<Window>) -> Result<(), String> {
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
