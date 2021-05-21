use anyhow::Result;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
use vekotin::geometry::line_segment::LineSegment2i;
use vekotin::geometry::triangle::{Triangle2i, Triangle3f};
use vekotin::geometry::Point2i;
use vekotin::gfx;
use vekotin::loader::obj::Obj;
use vekotin::math::Matrix3f;

pub struct Game {
    event_pump: sdl2::EventPump,
    canvas: Canvas<Window>,
    obj: Obj,
    angle: f32,
}

impl Game {
    pub fn new() -> Result<Self> {
        let sdl_context = sdl2::init().expect("failed to init SDL");
        let video_subsystem = sdl_context.video().expect("failed to get video context");

        let obj = Obj::from_file("assets/head.obj")?;

        // We create a window.
        let window = video_subsystem
            .window("sdl2 demo", 800, 800)
            .build()
            .expect("failed to build window");
        let mut canvas: Canvas<Window> = window
            .into_canvas()
            .build()
            .expect("failed to build window's canvas");

        let event_pump = sdl_context.event_pump().unwrap();

        Ok(Self {
            event_pump,
            canvas,
            obj,
            angle: 0.0,
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
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();

        let rot = Matrix3f::rotation_y(self.angle);
        gfx::cpu::draw_obj(&mut self.canvas, &self.obj, &rot);

        self.canvas.present();

        self.angle += 0.05;
        ::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 30));
        emscripten_main_loop::MainLoopEvent::Continue
    }
}

fn main() {
    let game = Game::new().unwrap();

    emscripten_main_loop::run(game);
}
