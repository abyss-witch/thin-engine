//! A thin (hence the name) game engine. Drawing done with `glium`, windowing done with `winit`
//! and input done with `winit-input-map`. Very simple to use with low boiler plate though it is
//! recomended to make your own structs to simplify drawing.
//! ```
//! use thin_engine::{prelude::*, meshes::screen};
//! use Action::*;
//!
//! #[derive(ToUsize)]
//! enum Action {
//!     Left,
//!     Right,
//!     Jump,
//!     Exit
//! }
//! let (event_loop, window, display) = thin_engine::set_up().unwrap();
//! let mut input = input_map!(
//!     (Left,  KeyCode::KeyA, MouseButton::Left, KeyCode::ArrowLeft),
//!     (Right, KeyCode::KeyD, MouseButton::Right, KeyCode::ArrowRight),
//!     (Jump,  KeyCode::KeyW, KeyCode::ArrowUp, KeyCode::Space),
//!     (Exit,  KeyCode::Escape)
//! );
//! let (box_indices, box_verts) = mesh!(
//!     &display, &screen::INDICES, &screen::VERTICES
//! );
//! let box_shader = Program::from_source(
//! &display, shaders::VERTEX, 
//! "#version 140
//! out vec4 colour;
//! void main() {
//!     colour = vec4(1.0, 0.0, 0.0, 1.0);
//! }", None).unwrap();
//! 
//! let mut player_pos = Vec2::ZERO;
//! let mut player_gravity = 0.0;
//! let mut player_can_jump = true;
//!
//! // camera matrix must be inverse
//! let camera = Mat4::from_scale(Vec3::splat(10.0)).inverse();
//! 
//! let target_delay = Duration::from_secs_f32(1.0/60.0); // target of 60 fps
//! let mut delta_time = 0.016; // change in time between frames
//! thin_engine::run(event_loop, &mut input, |input, target| {
//!     let elapsed = Instant::now();
//!     // set up frame
//!     let size = window.inner_size().into();
//!     display.resize(size);
//!     let mut frame = display.draw();
//!     let view2d = Mat4::view_matrix_2d(size);
//!     
//!     // game logic
//!     player_pos.x += input.axis(Right, Left) * 10.0 * delta_time;
//!     player_gravity += delta_time * 50.0;
//!     player_pos.y -= player_gravity * delta_time;
//!     if player_pos.y < 0.0 {
//!         player_pos.y = 0.0;
//!         player_can_jump = true;
//!     }
//!     if player_can_jump && input.pressed(Jump) {
//!         player_gravity = -20.0;
//!         player_can_jump = false;
//!     }
//!
//!     // draw
//!     frame.clear_color(0.0, 0.0, 0.0, 1.0);
//!     frame.draw(
//!         &box_verts, &box_indices, &box_shader, &uniform! {
//!             view: view2d, camera: camera,
//!             model: Mat4::from_pos(player_pos.extend(0.0)),
//!         }, &DrawParameters::default()
//!     );
//!     
//!     frame.finish().unwrap();
//!     if input.pressed(Exit) { target.exit() }
//!     let elapsed = elapsed.elapsed();
//!     thread::sleep(target_delay.saturating_sub(elapsed));
//!     delta_time = elapsed.max(target_delay).as_secs_f32();
//! }).unwrap();
//! ```

use glium::backend::glutin::SimpleWindowBuilder;
use winit::{
    error::EventLoopError,
    event::*,
    window::Window,
};
use winit_input_map::InputMap;
pub use glium;
pub use glium_types;
pub use winit;
pub use winit_input_map as input_map;

pub mod meshes;
pub mod shaders;

pub type Display = glium::Display<glium::glutin::surface::WindowSurface>;
pub type EventLoop = winit::event_loop::EventLoop<()>;
pub type WindowTarget = winit::event_loop::EventLoopWindowTarget<()>;

pub mod prelude {
    pub use glium::{
        draw_parameters, IndexBuffer,
        VertexBuffer, Program, Texture2d,
        uniform, Surface, Frame, DrawParameters
    };
    pub use std::time::{Duration, Instant};
    pub use std::thread;
    pub use glium_types::prelude::*;
    pub use crate::{meshes, shaders};
    pub use winit::event::MouseButton;
    pub use winit::keyboard::KeyCode;
    pub use winit::window::{Fullscreen, CursorGrabMode};
    pub use crate::input_map::*;
}
/// used to quickly set up thin engine.
pub fn set_up() -> Result<(EventLoop, Window, Display), EventLoopError> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let (window, display) = SimpleWindowBuilder::new().build(&event_loop);
    Ok((event_loop, window, display))
}
/// used to quickly set up logic. handles closed and input events for you. the `logic` var will be
/// run every frame.
/// ```
/// use thin_engine::prelude::*;
/// let (event_loop, window, display) = thin_engine::set_up().unwrap();
/// #[derive(ToUsize)]
/// enum Actions{
///     Debug
/// }
/// use Actions::*;
/// let mut input = input_map!(
///     (Debug, KeyCode::Space)
/// );
///
/// thin_engine::run(event_loop, &mut input, |_, _| {
///     let mut frame = display.draw();
///     frame.clear_color(0.0, 0.0, 0.0, 1.0);
///     frame.finish().unwrap();
/// });
/// ```
pub fn run<const BINDS: usize, F>(
    event_loop: EventLoop,
    input: &mut InputMap<BINDS>,
    mut logic: F,
) -> Result<(), EventLoopError>
where
    F: FnMut(&mut InputMap<BINDS>, &WindowTarget),
{
    event_loop.run(|event, target| {
        input.update(&event);
        match &event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => target.exit(),
            Event::AboutToWait => {
                logic(input, target);
                input.init();
            }
            _ => (),
        }
    })?;
    Ok(())
}
/// used to quickly set up logic. handles closed and input events for you. the `logic` var will be
/// run every frame. the `event_handler` var is
/// for if you want more control over the event handling and is run multiple times before logic.
/// ```
/// use thin_engine::prelude::*;
/// let (event_loop, window, display) = thin_engine::set_up().unwrap();
/// #[derive(ToUsize)]
/// enum Actions{
///     Debug
/// }
/// use Actions::*;
/// let mut input = input_map!(
///     (Debug, KeyCode::Space)
/// );
///
///
/// thin_engine::run_with_event_handler(
///     event_loop,
///     &mut input,
///     |event, target| {
///         match event {
///             //do something with events
///             _ => ()
///         }
///     }, |_|{
///         let mut frame = display.draw();
///         frame.clear_color(0.0, 0.0, 0.0, 1.0);
///         frame.finish().unwrap();
/// });
/// ```
pub fn run_with_event_handler<const BINDS: usize, F1, F2>(
    event_loop: EventLoop,
    input: &mut InputMap<BINDS>,
    mut event_handler: F2,
    mut logic: F1,
) -> Result<(), EventLoopError>
where
    F1: FnMut(&mut InputMap<BINDS>),
    F2: FnMut(&Event<()>, &WindowTarget),
{
    event_loop.run(|event, target| {
        input.update(&event);
        event_handler(&event, target);
        match &event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => target.exit(),
            Event::AboutToWait => {
                logic(input);
                input.init();
            }
            _ => (),
        }
    })?;
    Ok(())
}
/// resizable depth texture. recomended to  use with gliums `SimpleFrameBuffer` to draw onto a texture you can use
/// in another shader! usefull for fxaa
#[derive(Default)]
pub struct ResizableTexture2D {
    pub size: (u32, u32),
    pub texture: Option<glium::Texture2d>
}
impl ResizableTexture2D {
    pub fn resize(&mut self, display: &Display, new_size: (u32, u32)) {
        if self.size.0 != new_size.0 || self.size.1 != new_size.1 {
            self.texture = glium::Texture2d::empty(display, new_size.0, new_size.1).ok();
            self.size = new_size;
        }
    }
    pub fn resize_to_display(&mut self, display: &Display) {
        let new_size = display.get_framebuffer_dimensions();
        if self.size.0 != new_size.0 || self.size.1 != new_size.1 {
            self.texture = glium::Texture2d::empty(display, new_size.0, new_size.1).ok();
            self.size = new_size;
        }
    }
    /// borrows the texture or panics. to handle failed borrows use `self.texture.as_ref()` instead
    pub fn texture(&self) -> &glium::Texture2d {
        self.texture.as_ref().expect("texture was not initialised. maybe use 'new()' instead of 'default()'")
    }
    pub fn new(size: (u32, u32), display: &Display) -> Self {
        Self { size, texture: glium::Texture2d::empty(display, size.0, size.1).ok() }
    }
}
/// resizable depth texture. use with gliums `SimpleFrameBuffer::WithDepthTexture()` 
/// to create a texture you can draw on! usefull for things like fog and fxaa.
#[derive(Default)]
pub struct ResizableDepthTexture2D {
    size: (u32, u32),
    pub texture: Option<glium::texture::DepthTexture2d>
}
impl ResizableDepthTexture2D {
    pub fn resize(&mut self, display: &Display, new_size: (u32, u32)) {
        if self.size.0 != new_size.0 || self.size.1 != new_size.1 {
            self.texture = glium::texture::DepthTexture2d::empty(display, new_size.0, new_size.1).ok();
            self.size = new_size;
        }
    }
    /// borrows the texture or panics. to handle failed borrows use `self.texture.as_ref()` instead
    pub fn texture(&self) -> &glium::texture::DepthTexture2d {
        self.texture.as_ref().expect("texture was not initialised. maybe use 'new()' instead of 'default()'")
    }
    pub fn new(size: (u32, u32), display: &Display) -> Self {
        Self { size, texture: glium::texture::DepthTexture2d::empty(display, size.0, size.1).ok() }
    }
    pub fn resize_to_display(&mut self, display: &Display) {
        let new_size = display.get_framebuffer_dimensions();
        if self.size.0 != new_size.0 || self.size.1 != new_size.1 {
            self.texture = glium::texture::DepthTexture2d::empty(display, new_size.0, new_size.1).ok();
            self.size = new_size;
        }
    }
}
