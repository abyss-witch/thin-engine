use glium::glutin::surface::WindowSurface;
use glium::{backend::glutin::SimpleWindowBuilder, Display};
use winit::{
    error::EventLoopError,
    event::*,
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::Window,
};
use winit_input_map::Input;

pub use glium;
pub use glium_types;
pub use winit;
pub use winit_input_map as input_map;

pub mod prelude {
    pub use glium::{uniform, DrawParameters, Program, Surface};
    pub use glium_types::prelude::*;
    pub use winit::keyboard::KeyCode;
    pub use winit_input_map::*;
}

///used to quickly set up thin engine.
pub fn set_up() -> Result<(EventLoop<()>, Window, Display<WindowSurface>), EventLoopError> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let (window, display) = SimpleWindowBuilder::new().build(&event_loop);
    Ok((event_loop, window, display))
}
/// used to quickly set up logic. handles closed and input events for you. the `logic` var will be
/// run every frame. the `event_handler` var is
/// for if you want more control over the event handling.
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
/// run(event_loop, &mut input, |_, _| (), |_|{
///     let mut frame = display.draw();
///     frame.clear_color(0, 0, 0, 0);
///     frame.finish().unwrap();
/// });
/// ```
pub fn run<const BINDS: usize, F1, F2>(
    event_loop: EventLoop<()>,
    input: &mut Input<BINDS>,
    mut event_handler: F2,
    mut logic: F1,
) -> Result<(), EventLoopError>
where
    F1: FnMut(&mut Input<BINDS>),
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
                input.init()
            }
            _ => (),
        }
    })?;
    Ok(())
}
