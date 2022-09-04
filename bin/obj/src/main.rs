use anyhow::Result;
use geometry::transform::Transform;
use geometry::Point3f;
use gfx::camera::Camera;
use loader::obj::Obj;
use loader::png::Png;
use math::Vec3f;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::TimerSubsystem;
use std::f32::consts::FRAC_PI_2;

pub struct Game {
    event_pump: sdl2::EventPump,
    canvas: Canvas<Window>,
    camera: Camera,
    timer: TimerSubsystem,
    ticks: u32,
    obj: Obj,
    texture: Png,
    angle: f32,
    rotating: bool,
}

static WIDTH: u32 = 1200;
static HEIGHT: u32 = 1200;

impl Game {
    pub fn new() -> Result<Self> {
        let sdl_context = sdl2::init().expect("failed to init SDL");
        let video_subsystem = sdl_context.video().expect("failed to get video context");
        let timer = sdl_context.timer().expect("failed to get timer subsystem");

        let obj = Obj::from_file("assets/head.obj")?;
        let texture = Png::from_file("assets/head_diffuse.png")?;
        // We create a window.
        let window = video_subsystem
            .window("sdl2 demo", WIDTH, HEIGHT)
            .build()
            .expect("failed to build window");
        let canvas: Canvas<Window> = window
            .into_canvas()
            .build()
            .expect("failed to build window's canvas");
        let mut camera = Camera {
            xform: Transform::translation(Vec3f::new(-10.0, 0.0, 0.0)),
            projection: Transform::infinite_projection(1.0, 1.0, 0.1, 0.001),
        };
        camera.look_at(Point3f::new(0., 0., 0.));

        let event_pump = sdl_context.event_pump().unwrap();

        Ok(Self {
            event_pump,
            canvas,
            camera,
            timer,
            ticks: 0,
            obj,
            texture,
            angle: 0.0,
            rotating: true,
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
                Event::KeyUp {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    self.rotating = !self.rotating;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    self.camera.truck(0.1);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    self.camera.truck(-0.1);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    self.camera.dolly(0.1);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    self.camera.dolly(-0.1);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    self.camera.look_at(Point3f::new(0., 0., 0.));
                }
                _ => {}
            }
        }

        let object_transform = Transform::rotation_z(self.angle) * Transform::rotation_x(FRAC_PI_2);
        let view = self.camera.view().unwrap() * object_transform;

        let texture_creator = self.canvas.texture_creator();
        let mut texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGB24, WIDTH, HEIGHT)
            .expect("Failed to create texture");

        texture
            .with_lock(None, |buffer: &mut [u8], _pitch: usize| {
                let viewport = self.canvas.viewport();
                let mut canvas = gfx::cpu::canvas::Canvas {
                    buffer,
                    width: viewport.width(),
                    height: viewport.height(),
                };
                gfx::cpu::draw_obj(
                    &mut canvas,
                    &self.obj,
                    &self.texture,
                    view,
                    self.camera.projection,
                );
            })
            .expect("Failed to render on texture");

        self.canvas.clear();
        self.canvas
            .copy(&texture, None, None)
            .expect("Failed to copy texture to canvas");
        self.canvas.present();

        if self.rotating {
            self.angle += 0.005;
        }
        let ticks = self.timer.ticks();
        let delta = ticks - self.ticks;
        println!("{}", delta);
        self.ticks = ticks;

        // ::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 30));
        emscripten_main_loop::MainLoopEvent::Continue
    }
}

fn main() {
    let game = Game::new().unwrap();

    emscripten_main_loop::run(game);
}
