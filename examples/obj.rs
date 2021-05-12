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

pub struct Game {
    event_pump: sdl2::EventPump,
    _canvas: Canvas<Window>,
}

fn draw_triangle(canvas: &mut Canvas<Window>, obj: &Obj, i: usize) {
    let v_indices = &obj.vertex_index_triples[i];
    let viewport = canvas.viewport();
    let w = viewport.width();
    let h = viewport.height();

    let v0 = obj.vertices[v_indices.0 as usize];
    // Project the 3D points onto the canvas, orthographic projection
    let p0 = Point2i::new(
        ((v0.x() + 1.0) * w as f32 / 2.0) as i32,
        h as i32 - ((v0.y() + 1.0) * h as f32 / 2.0) as i32,
    );
    let v1 = obj.vertices[v_indices.1 as usize];
    let p1 = Point2i::new(
        ((v1.x() + 1.0) * w as f32 / 2.0) as i32,
        h as i32 - ((v1.y() + 1.0) * h as f32 / 2.0) as i32,
    );
    let v2 = obj.vertices[v_indices.2 as usize];
    let p2 = Point2i::new(
        ((v2.x() + 1.0) * w as f32 / 2.0) as i32,
        h as i32 - ((v2.y() + 1.0) * h as f32 / 2.0) as i32,
    );
    let normal = Triangle3f::new(&v0.into(), &v1.into(), &v2.into())
        .normal()
        .unit();

    let t = Triangle2i::new(&p0, &p1, &p2);

    let white = Color::RGBA(255, 255, 255, 255);

    if normal.z() >= 0.0 {
        let intensity = (normal.z() * 255.0) as u8;
        let c = Color::RGBA(intensity, intensity, intensity, 255);
        gfx::cpu::draw_triangle(canvas, &t, c);
        gfx::cpu::draw_line_segment(canvas, &LineSegment2i::new(&p0, &p1), white);
        gfx::cpu::draw_line_segment(canvas, &LineSegment2i::new(&p1, &p2), white);
        gfx::cpu::draw_line_segment(canvas, &LineSegment2i::new(&p2, &p0), white);
    }
}

impl Game {
    pub fn new() -> Result<Self> {
        let sdl_context = sdl2::init().expect("failed to init SDL");
        let video_subsystem = sdl_context.video().expect("failed to get video context");

        let obj = Obj::from_file("assets/head.obj")?;

        // We create a window.
        let window = video_subsystem
            .window("sdl2 demo", 800, 600)
            .build()
            .expect("failed to build window");
        let mut canvas: Canvas<Window> = window
            .into_canvas()
            .build()
            .expect("failed to build window's canvas");

        for i in 0..obj.vertex_index_triples.len() {
            draw_triangle(&mut canvas, &obj, i);
        }
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
