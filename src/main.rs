extern crate anyhow;
extern crate sdl2;
extern crate gl;

use anyhow::Result;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;

mod loader;

pub struct Game {
    window: sdl2::video::Window,
    event_pump: sdl2::EventPump,
    _gl_ctx: sdl2::video::GLContext,
}

impl Game {
    pub fn new() -> Result<Self> {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        
        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_version(3, 3);
    
        let window = video_subsystem.window("Window", 800, 600)
            .opengl()
            .build()?;
    
        let _gl_ctx = window.gl_create_context().unwrap();
        gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);
        
        debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
        debug_assert_eq!(gl_attr.context_version(), (3, 3));

        let event_pump = sdl_context.event_pump().unwrap();
        
        Ok(Self { window, event_pump, _gl_ctx })
    }  
}

impl emscripten_main_loop::MainLoop for Game {
    fn main_loop(&mut self) -> emscripten_main_loop::MainLoopEvent {
        unsafe {
            gl::ClearColor(0.6, 0.0, 0.8, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        self.window.gl_swap_window();
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    return emscripten_main_loop::MainLoopEvent::Terminate;
                },
                _ => {}
            }
        }
        ::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 60));
        emscripten_main_loop::MainLoopEvent::Continue
    }
}

fn main() {
    let game = Game::new().unwrap();
    loader::png::load_from_file("assets/test.png").unwrap();

    emscripten_main_loop::run(game);
}