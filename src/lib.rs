use glium::backend::glutin::SimpleWindowBuilder;
use winit::{
    error::EventLoopError,
    event::*,
    event_loop::EventLoopWindowTarget,
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

pub mod prelude {
    pub use glium::{
        draw_parameters, IndexBuffer,
        VertexBuffer, Program, Texture2d,
        uniform, Surface, Frame, DrawParameters
    };
    pub use glium_types::prelude::*;
    pub use crate::{meshes, shaders};
    pub use winit::keyboard::KeyCode;
    pub use winit::event::MouseButton;
    pub use winit::window::{Fullscreen, CursorGrabMode};
    pub use crate::input_map::*;
}
///used to quickly set up thin engine.
pub fn set_up() -> Result<(EventLoop, Window, Display), EventLoopError> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let (window, display) = SimpleWindowBuilder::new().build(&event_loop);
    Ok((event_loop, window, display))
}
/// used to quickly set up logic. handles closed and input events for you. the `logic` var will be
/// run every frame.
/// ```
/// let (event_loop, window, display) = thin_engine::set_up();
///
/// enum Actions{
///     Debug
/// }
/// impl Into<usize> for Actions{
///     fn into(self) -> usize{
///         self as usize
///     }
/// }
/// use Actions::*;
/// let mut input = thin_engine::Input::new([
///     (vec![InputCode::keycode(KeyCode::Space), Debug)
/// ]);
///
/// run(event_loop, &mut input, |_|{
///     let mut frame = display.draw();
///     frame.clear_color(0, 0, 0, 0);
///     frame.finish().unwrap();
/// });
/// ```
pub fn run<const BINDS: usize, F1>(
    event_loop: EventLoop,
    input: &mut InputMap<BINDS>,
    mut logic: F1,
) -> Result<(), EventLoopError>
where
    F1: FnMut(&mut InputMap<BINDS>),
{
    event_loop.run(|event, target| {
        input.update(&event);
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
/// used to quickly set up logic. handles closed and input events for you. the `logic` var will be
/// run every frame. the `event_handler` var is
/// for if you want more control over the event handling and is run multiple times before logic.
/// ```
/// let (event_loop, window, display) = thin_engine::set_up();
///
/// enum Actions{
///     Debug
/// }
/// impl Into<usize> for Actions{
///     fn into(self) -> usize{
///         self as usize
///     }
/// }
/// use Actions::*;
/// let mut input = thin_engine::Input::new([
///     (vec![InputCode::keycode(KeyCode::Space), Debug)
/// ]);
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
///         frame.clear_color(0, 0, 0, 1);
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
    F2: FnMut(&Event<()>, &EventLoopWindowTarget<()>),
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
/// to create a texture you can draw on! usefull for things like fog.
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
