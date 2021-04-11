use anyhow::{bail, Result};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::{BlendMode, Canvas};
use sdl2::video::Window;
use vekotin::geometry::line_segment::{LineSegment, LineSegment2i};
use vekotin::geometry::Point2i;
use vekotin::gfx;
use vekotin::loader::png;

pub struct Game {
    event_pump: sdl2::EventPump,
    _canvas: Canvas<Window>,
}

fn pixel_format(image: &png::Png) -> Result<PixelFormatEnum> {
    use png::BitDepth::*;
    use png::ColorType::*;
    match (&image.bit_depth, &image.color_type) {
        (Bits8, RGB) => Ok(PixelFormatEnum::RGB24),
        (Bits8, RGBA) => Ok(PixelFormatEnum::RGBA32),
        (bpp, ct) => bail!("Can't handle these: ({:?}, {:?}", bpp, ct),
    }
}

impl Game {
    pub fn new() -> Result<Self> {
        let sdl_context = sdl2::init().expect("failed to init SDL");
        let video_subsystem = sdl_context.video().expect("failed to get video context");

        let img = png::from_file("assets/test.png")?;
        println!("{}", img.bytes_per_pixel);
        // We create a window.
        let window = video_subsystem
            .window("sdl2 demo", img.width, img.height)
            .build()
            .expect("failed to build window");

        // We get the canvas from which we can get the `TextureCreator`.
        let mut canvas: Canvas<Window> = window
            .into_canvas()
            .build()
            .expect("failed to build window's canvas");
        let texture_creator = canvas.texture_creator();

        let px_fmt = pixel_format(&img)?;
        println!("{:?}", px_fmt);
        let mut texture =
            texture_creator.create_texture_streaming(px_fmt, img.width, img.height)?;
        texture.set_blend_mode(BlendMode::Blend);
        texture.update(None, &img.data, (img.bytes_per_pixel * img.width) as usize)?;

        canvas.copy(&texture, None, None).unwrap();
        gfx::cpu::draw_line_segment(
            &mut canvas,
            &LineSegment2i::new(Point2i::new(0, 0), Point2i::new(200, 200)),
            Color::RGBA(255, 255, 255, 255),
        );
        canvas.present();

        let event_pump = sdl_context.event_pump().unwrap();

        Ok(Self {
            event_pump,
            _canvas: canvas,
        })
    }
}

impl emscripten_main_loop::MainLoop for Game {
    fn main_loop(&mut self) -> emscripten_main_loop::MainLoopEvent {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    return emscripten_main_loop::MainLoopEvent::Terminate;
                }
                _ => {}
            }
        }
        ::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 60));
        emscripten_main_loop::MainLoopEvent::Continue
    }
}

fn main() {
    let game = Game::new().unwrap();

    emscripten_main_loop::run(game);
}
