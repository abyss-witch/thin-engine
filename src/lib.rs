//! A thin game engine (hence the name). Drawing done with `glium`, game variables done with
//! `glium-types`, windowing done with `winit` and input done with `winit-input-map`. It has easy fxaa
//! support and low boilerplate despite having lots of control.
//! ```
//! use thin_engine::{prelude::*, meshes::screen};
//! use Action::*;
//!
//! #[derive(Hash, PartialEq, Eq, Clone, Copy)]
//! enum Action {
//!     Left,
//!     Right,
//!     Jump,
//!     Exit
//! }
//! let (event_loop, window, display) = thin_engine::set_up().unwrap();
//! let mut input = {
//!     use thin_engine::input_map_setup::*;
//!     input_map!(
//!         (Left,  KeyA, MouseButton::Left,  ArrowLeft,  GamepadButton::DPadLeft),
//!         (Right, KeyD, MouseButton::Right, ArrowRight, GamepadButton::DPadRight),
//!         (Jump,  KeyW, ArrowUp, Space, GamepadButton::South),
//!         (Exit,  Escape, GamepadButton::Start)
//!     )
//! };
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
//! let settings = Settings::from_fps(60); // target of 60 fps
//! let mut delta_time = 0.016; // change in time between frames
//! thin_engine::run(event_loop, input, settings, |input, _settings, target| {
//!     let frame_start = Instant::now();
//!     // set up frame
//!     let size = window.inner_size().into();
//!     display.resize(size);
//!     let mut frame = display.draw();
//!     let view2d = Mat4::view_matrix_2d(size);
//!     
//!     if input.pressed(Exit) { target.exit() }
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
//!     frame.finish().unwrap();
//!     delta_time = frame_start.elapsed().as_secs_f32();
//! }).unwrap();
//! ```

#![allow(deprecated)]
use glium::backend::glutin::SimpleWindowBuilder;
use winit::{
    error::EventLoopError,
    event::*,
    window::Window,
};
use std::{hash::Hash};
pub use gilrs;
use gilrs::Gilrs;
use winit_input_map::InputMap;
pub use glium;
pub use glium_types;
pub use winit;
pub use winit_input_map as input_map;
use std::time::{Duration, Instant};

/// run time settings for thin engine including gamepad settings (through gilrs) and fps settings.
/// when running `default()` the gamepads may fail to initialise and the program will continue
/// running after printing the error. if this is undesirable use `with_gamepads()` instead.
pub struct Settings {
    gamepads: Option<Gilrs>,
    min_frame_duration: Option<Duration>
}
impl Settings {
    pub fn new(gamepads: Option<Gilrs>, min_frame_duration: Option<Duration>) -> Self {
        Self { gamepads, min_frame_duration }
    }
    /// creates settings with the minimum frame duration set to 1/fps.
    pub fn from_fps(fps: u32) -> Self {
        let gamepads = Gilrs::new().map_err(|i| println!("{i}")).ok();
        let min_frame_duration = Some(Duration::from_secs_f32(1.0/fps as f32));

        Self::new(gamepads, min_frame_duration)
    }
    /// guarantees gamepads will be set instead of printing an error and moving on.
    pub fn with_gamepads() -> Result<Self, gilrs::Error> {
        let gilrs = Gilrs::new()?;
        Ok(Self::new(Some(gilrs), None))
    }
    /// sets the minimum frame duration to 1/fps or none if inputed.
    pub fn set_target_fps(&mut self, target_fps: Option<u32>) {
        let min_duration = target_fps.map(|i| Duration::from_secs_f32(1.0/i as f32));
        self.min_frame_duration = min_duration;
    }
}
impl Default for Settings {
    fn default() -> Self {
        let gamepads = Gilrs::new().map_err(|i| println!("{i}")).ok();
        Self::new(gamepads, None)
    }
}
pub mod meshes;
pub mod shaders;

pub type Display = glium::Display<glium::glutin::surface::WindowSurface>;
pub type EventLoop = winit::event_loop::EventLoop<()>;
pub type WindowTarget = winit::event_loop::ActiveEventLoop;

pub mod prelude {
    pub use glium::{
        draw_parameters, IndexBuffer, self,
        VertexBuffer, Program, Texture2d,
        uniform, Surface, Frame, DrawParameters
    };
    pub use crate::Settings;
    pub use std::time::{Duration, Instant};
    pub use std::thread;
    pub use glium_types::prelude::*;
    pub use crate::{meshes, shaders};
    pub use winit::event::MouseButton;
    pub use winit::keyboard::KeyCode;
    pub use gilrs::ev::{Button as GamepadButton, Axis as GamepadAxis};
    pub use winit::window::{Fullscreen, CursorGrabMode};
    pub use crate::input_map::*;
}
/// imports base roots of input_code options to reduce boilerplate
/// ```
/// #[derive(Hash, PartialEq, Eq, Clone, Copy)]
/// enum Actions { Foo, Bar }
/// use Actions::*;
/// let input_map = {
///     use thin_engine::input_map_setup::*;
///     thin_engine::input_map::input_map!(
///         (Foo, Axis(LeftStickX, Pos), MouseMoveX(Pos),  GamepadButton::West),
///         (Bar, MouseScroll(Neg),      MouseButton::Left)
///     )
/// };
/// ```
pub mod input_map_setup {
    pub use winit::keyboard::KeyCode::*;
    pub use winit::event::MouseButton;
    pub use gilrs::Axis::*;
    pub use winit_input_map::{GamepadButton, *};
    pub use winit_input_map::{DeviceInput::*, GamepadInput::Axis, AxisSign::*};
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
/// #[derive(Hash, PartialEq, Eq, Clone, Copy)]
/// enum Actions{ Debug }
/// use Actions::*;
/// let mut input = input_map!(
///     (Debug, KeyCode::Space, GamepadButton::South)
/// );
/// let settings = Settings::default();
/// thin_engine::run(event_loop, input, settings, |_, _, _| {
///     let mut frame = display.draw();
///     frame.clear_color(0.0, 0.0, 0.0, 1.0);
///     frame.finish().unwrap();
/// }).unwrap();
/// ```
pub fn run<T, F>(
    event_loop: EventLoop,
    mut input: InputMap<T>,
    mut settings: Settings,
    mut logic: F,
) -> Result<(), EventLoopError>
where
    T: Hash + PartialEq + Eq + Copy + Clone,
    F: FnMut(&mut InputMap<T>, &mut Settings, &WindowTarget),
{
    let mut frame_time = Instant::now();
    event_loop.run(|event, target| {
        if let Some(ref mut gilrs) = settings.gamepads { input.update_with_gilrs(gilrs) }
        match &event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => target.exit(),
            Event::WindowEvent { event, .. } => input.update_with_window_event(event),
            Event::DeviceEvent { event, .. } => input.update_with_device_event(event),
            Event::AboutToWait => {
                let update = settings.min_frame_duration
                    .map(|i| i >= frame_time.elapsed())
                    .unwrap_or(true);
                if update {
                    logic(&mut input, &mut settings, target);
                    frame_time = Instant::now();
                    input.init() }
            },
            _ => (),
        }
    })
}
/// used to quickly set up logic. handles closed and input events for you. the `logic` var will be
/// run every frame. the `event_handler` var is
/// for if you want more control over the event handling and is run multiple times before logic.
/// ```
/// use thin_engine::prelude::*;
/// let (event_loop, window, display) = thin_engine::set_up().unwrap();
/// #[derive(Hash, PartialEq, Eq, Clone, Copy)]
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
///     event_loop, input,
///     Settings::default(),
///     |event, input, settings, target| {
///         match event {
///             //do something with events
///             _ => ()
///         }
///     }, |input, settings, target|{
///         let mut frame = display.draw();
///         frame.clear_color(0.0, 0.0, 0.0, 1.0);
///         frame.finish().unwrap();
/// });
/// ```
pub fn run_with_event_handler<T, F1, F2>(
    event_loop: EventLoop,
    mut input: InputMap<T>,
    mut settings: Settings,
    mut event_handler: F2,
    mut logic: F1,
) -> Result<(), EventLoopError>
where
    T: Hash + PartialEq + Eq + Clone + Copy,
    F1: FnMut(&mut InputMap<T>, &mut Settings, &WindowTarget),
    F2: FnMut(&Event<()>, &mut InputMap<T>, &mut Settings, &WindowTarget),
{
    let mut frame_time = Instant::now();
    event_loop.run(|event, target| {
        if let Some(ref mut gilrs) = settings.gamepads { input.update_with_gilrs(gilrs) }
        event_handler(&event, &mut input, &mut settings, target);
        match &event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => target.exit(),
            Event::WindowEvent { event, .. } => input.update_with_window_event(event),
            Event::DeviceEvent { event, .. } => input.update_with_device_event(event),
            Event::AboutToWait => {
                let update = settings.min_frame_duration
                    .map(|i| i >= frame_time.elapsed())
                    .unwrap_or(true);
                if update {
                    logic(&mut input, &mut settings, target);
                    frame_time = Instant::now();
                    input.init()
                }
            },
            _ => (),
        }
    })
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
