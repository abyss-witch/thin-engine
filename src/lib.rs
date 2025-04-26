//! A thin game engine (hence the name). Drawing done with `glium`, game variables done with
//! `glium-types`, windowing done with `winit` and input done with `winit-input-map`. It has easy fxaa
//! support and low boilerplate despite having lots of control.
//! ```
//! use thin_engine::{prelude::*, meshes::screen};
//! use std::{cell::RefCell, rc::Rc};
//! use Action::*;
//!
//! #[derive(Hash, PartialEq, Eq, Clone, Copy)]
//! enum Action {
//!     Left,
//!     Right,
//!     Jump,
//!     Exit
//! }
//! let event_loop = EventLoop::new().unwrap();
//! event_loop.set_control_flow(ControlFlow::Poll);
//! let mut input = { use base_input_codes::*; input_map!(
//!     (Left,  KeyA, MouseButton::Left,  ArrowLeft,  DPadLeft),
//!     (Right, KeyD, MouseButton::Right, ArrowRight, DPadRight),
//!     (Jump,  KeyW, ArrowUp, Space, GamepadInput::South),
//!     (Exit,  Escape, GamepadInput::Start)
//! )};
//!
//! struct Graphics {
//!     box_indices: IndexBuffer<u32>,
//!     box_vertices: VertexBuffer<Vertex>,
//!     box_uvs: VertexBuffer<TextureCoords>,
//!     box_normals: VertexBuffer<Normal>,
//!     box_shader: Program
//! }
//! let graphics: Rc<RefCell<Option<Graphics>>> = Rc::default();
//! let graphics_setup = graphics.clone();
//!
//! let mut player_pos = Vec2::ZERO;
//! let mut player_gravity = 0.0;
//! let mut player_can_jump = true;
//!
//! // camera matrix must be inverse
//! let camera = Mat4::from_scale(Vec3::splat(10.0)).inverse();
//! 
//! let settings = Settings::from_fps(60); // target of 60 fps
//! let mut frame_start = Instant::now();
//! thin_engine::builder(input).with_setup(move |display, window, _| {
//!     // some computers will panic when a vertex buffer is used but not passed a value. so we must
//!     // initialise empty vertex buffers.
//!     let (box_indices, box_vertices, box_uvs, box_normals) = mesh!(
//!         display, &screen::INDICES, &screen::VERTICES,
//!         &[] as &[TextureCoords; 0], &[] as &[Normal; 0]
//!     );
//!     let box_shader = Program::from_source(
//!         display, shaders::VERTEX, 
//!         "#version 140
//!         out vec4 colour;
//!         void main() {
//!             colour = vec4(1.0, 0.0, 0.0, 1.0);
//!         }", None
//!     ).unwrap();
//!     graphics_setup.replace(Some(Graphics {
//!         box_vertices, box_uvs, box_normals, box_indices, box_shader
//!     }));
//! }).with_update(move |input, display, _settings, target, window| {
//!     // gets time between frames
//!     let delta_time = frame_start.elapsed().as_secs_f32();
//!     frame_start = Instant::now();
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
//!     let graphics = graphics.borrow();
//!     let Graphics {
//!         box_vertices, box_uvs, box_normals, box_indices, box_shader
//!     } = graphics.as_ref().unwrap();
//!     // set up frame
//!     let mut frame = display.draw();
//!     let view2d = Mat4::view_matrix_2d(frame.get_dimensions());
//!
//!     // draw
//!     frame.clear_color(0.0, 0.0, 0.0, 1.0);
//!     frame.draw(
//!         (box_vertices, box_uvs, box_normals), box_indices,
//!         box_shader, &uniform! {
//!             view: view2d, camera: camera,
//!             model: Mat4::from_pos(player_pos.extend(0.0)),
//!         }, &DrawParameters::default()
//!     );
//!     window.pre_present_notify();
//!     frame.finish().unwrap();
//! }).with_settings(Settings::from_fps(60))
//!     .build(event_loop).unwrap();
//! ```

#![allow(deprecated)]
use glium::backend::glutin::SimpleWindowBuilder;
pub use gilrs;
use gilrs::Gilrs;
pub use glium;
pub use glium_types;
pub use winit;
pub use winit_input_map as input_map;
use std::time::Duration;

/// run time settings for thin engine including gamepad settings (through gilrs) and fps settings.
/// when running `default()` the gamepads may fail to initialise and the program will continue
/// running after printing the error. if this is undesirable use `with_gamepads()` instead.
pub struct Settings {
    pub gamepads: Option<Gilrs>,
    pub min_frame_duration: Option<Duration>
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
    pub fn get_fps(&self) -> Option<u32> {
        self.min_frame_duration.map(|i| (1.0 / i.as_secs_f64()).round() as u32)
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
pub mod application;
#[cfg(feature = "text")]
pub mod text_renderer;

pub type Display = glium::Display<glium::glutin::surface::WindowSurface>;

use winit_input_map::InputMap;
use crate::application::ThinBuilder;
use std::hash::Hash;
pub fn builder<'a, H: Hash + Eq + Copy>(input_map: InputMap<H>) -> ThinBuilder<'a, H> {
    ThinBuilder::<'a, H>::new(input_map)
}

pub mod prelude {
    pub use crate::application::*;
    pub use glium::{
        draw_parameters, IndexBuffer, self,
        VertexBuffer, Program, Texture2d,
        uniform, Surface, Frame, DrawParameters,
        backend::glutin::simple_window_builder::SimpleWindowBuilder
    };
    pub use crate::Settings;
    pub use std::time::{Duration, Instant};
    pub use std::thread;
    pub use glium_types::prelude::*;
    pub use crate::{meshes, shaders};
    pub use winit::{self, event::MouseButton, keyboard::KeyCode};
    pub use gilrs::ev::{Button as GamepadButton, Axis as GamepadAxis};
    pub use winit::{event_loop::*, window::{Fullscreen, CursorGrabMode}};
    pub use crate::input_map::*;
}
/// resizable depth texture. recomended to  use with gliums `SimpleFrameBuffer` to draw onto a texture you can use
/// in another shader! usefull for fxaa
#[derive(Default)]
pub struct ResizableTexture2d {
    pub size: (u32, u32),
    pub texture: Option<glium::Texture2d>
}
impl ResizableTexture2d {
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
pub struct ResizableDepthTexture2d {
    size: (u32, u32),
    pub texture: Option<glium::texture::DepthTexture2d>
}
impl ResizableDepthTexture2d {
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
