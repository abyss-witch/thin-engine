#![allow(clippy::type_complexity)]
use winit::{
    application::ApplicationHandler,
    window::{Window, WindowId},
    event_loop::{EventLoop, ActiveEventLoop},
    event::*
};
use crate::{SimpleWindowBuilder, Display, Settings};
use std::{hash::Hash, time::Instant};
use winit_input_map::InputMap;
/// holds all the data and runs the application.
pub struct ThinEngine<'a, H, D, S, U, E>
where H: Hash + PartialEq + Eq + Clone + Copy,
S: FnMut(&Display, &mut Window, &ActiveEventLoop),
U: FnMut(&mut InputMap<H>, &Display, &mut Settings, &ActiveEventLoop, &mut Window),
D: FnMut(&mut InputMap<H>, &Display, &mut Settings, &ActiveEventLoop, &mut Window),
E: FnMut(Event<()>, &ActiveEventLoop, Option<&(Window, Display)>)
{
    state: Option<(Window, Display)>,
    window_settings: Option<SimpleWindowBuilder>,
    update:        &'a mut U,
    draw:          &'a mut D,
    setup:         &'a mut S,
    event_handler: &'a mut E,
    input_map: InputMap<H>,
    settings: Settings,
    frame_start: Instant,
}
impl<H, D, S, U, E> ApplicationHandler for ThinEngine<'_, H, D, S, U, E>
where H: Hash + PartialEq + Eq + Clone + Copy,
S: FnMut(&Display, &mut Window, &ActiveEventLoop),
U: FnMut(&mut InputMap<H>, &Display, &mut Settings, &ActiveEventLoop, &mut Window),
D: FnMut(&mut InputMap<H>, &Display, &mut Settings, &ActiveEventLoop, &mut Window),
E: FnMut(Event<()>, &ActiveEventLoop, Option<&(Window, Display)>)
{
   fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() { return }
        let (mut window, display) = self.window_settings
            .take().expect("No window settings are available")
            .build(event_loop);
       (self.setup)(&display, &mut window, event_loop);
        self.state = Some((window, display));
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::Resized(size) => self.state.as_mut().unwrap().1.resize(size.into()),
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                let Some((ref mut window, ref display)) = self.state else { return };
                let input = &mut self.input_map;
                let settings = &mut self.settings;
                (self.draw)(input, display, settings, event_loop, window)
            },
            _ => self.input_map.update_with_window_event(&event)
        }
        (self.event_handler)(Event::WindowEvent { window_id, event }, event_loop, self.state.as_ref());
    }
    fn device_event(&mut self, event_loop: &ActiveEventLoop, device_id: DeviceId, event: DeviceEvent) {
        self.input_map.update_with_device_event(device_id, &event);
        (self.event_handler)(Event::DeviceEvent { device_id, event }, event_loop, self.state.as_ref());
    }
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(ref mut gilrs) = self.settings.gamepads { self.input_map.update_with_gilrs(gilrs) }

        let update = self.settings.min_frame_duration
            .map(|i| i <= self.frame_start.elapsed())
            .unwrap_or(true);
        if update {
            let (window, display) = self.state.as_mut().unwrap();
            (self.update)(&mut self.input_map, display, &mut self.settings, event_loop, window);
            self.frame_start = Instant::now();
            self.input_map.init();
        }
        (self.event_handler)(Event::AboutToWait, event_loop, self.state.as_ref());
    }
    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        let window = self.state.take().unwrap().0;

        self.window_settings = Some(SimpleWindowBuilder::new()
            .set_window_builder(Window::default_attributes()
            .with_inner_size(window.inner_size())
            // todo position
            .with_resizable(window.is_resizable())
            .with_enabled_buttons(window.enabled_buttons())
            .with_title(window.title())
            .with_fullscreen(window.fullscreen())
            .with_maximized(window.is_maximized())
            .with_visible(window.is_visible().unwrap_or(true))
            // todo transparent
            .with_decorations(window.is_decorated())
            .with_theme(window.theme())
            // todo resize increments
            // todo parrent window
        ));
        (self.event_handler)(Event::Suspended, event_loop, self.state.as_ref());
    }
}
/// holds data used to build and run the program
pub struct ThinBuilder<'a, H: Hash + PartialEq + Eq + Clone + Copy> {
    window_settings: SimpleWindowBuilder,
    input_map: InputMap<H>,
    settings: Settings,
    update: Box<dyn FnMut(&mut InputMap<H>, &Display, &mut Settings, &ActiveEventLoop, &mut Window) + 'a>,
    setup: Box<dyn FnMut(&Display, &mut Window, &ActiveEventLoop) + 'a>,
    draw: Box<dyn FnMut(&mut InputMap<H>, &Display, &mut Settings, &ActiveEventLoop, &mut Window) + 'a>,
    event_handler: Box<dyn FnMut(Event<()>, &ActiveEventLoop, Option<&(Window, Display)>) + 'a>
}
impl<'a, H: Hash + PartialEq + Eq + Clone + Copy> ThinBuilder<'a, H> {
   pub fn new(input_map: InputMap<H>) -> ThinBuilder<'a, H> {
        ThinBuilder {
            input_map,
            settings: Settings::default(),
            update:        Box::new(|_, _, _, _, _| {}),
            setup:         Box::new(|_, _, _|       {}),
            draw:          Box::new(|_, _, _, _, _| {}),
            event_handler: Box::new(|_, _, _|       {}),
            window_settings: SimpleWindowBuilder::new()
        }
    }
    /// builds and runs the program
    pub fn build(mut self, ev: EventLoop<()>) -> Result<(), winit::error::EventLoopError> {
        let mut engine = ThinEngine {
            state: None,
            window_settings: Some(self.window_settings),
            update:        &mut self.update,
            draw:          &mut self.draw,
            setup:         &mut self.setup,
            event_handler: &mut self.event_handler,
            input_map: self.input_map,
            settings:  self.settings,
            frame_start: Instant::now(),
        };
        ev.run_app(&mut engine)
    }
    /// this is run whenever a draw request is scheduled
    pub fn with_draw(
        mut self,
        draw: impl FnMut(&mut InputMap<H>, &Display, &mut Settings, &ActiveEventLoop, &mut Window) + 'a
    ) -> Self {
        self.draw = Box::new(draw);
        self
    }
    /// this is run whenever the window and display are created
    pub fn with_setup(mut self, setup: impl FnMut(&Display, &mut Window, &ActiveEventLoop) + 'a) -> Self {
        self.setup = Box::new(setup);
        self
    }
    /// this is run always. if min_duration in settings is set, then it is throttled to said value
    pub fn with_update(
        mut self,
        update: impl FnMut(&mut InputMap<H>, &Display, &mut Settings, &ActiveEventLoop, &mut Window) + 'a
    ) -> Self {
        self.update = Box::new(update);
        self
    }
    pub fn with_event_handler(
        mut self,
        event_handler: impl FnMut(Event<()>, &ActiveEventLoop, Option<&(Window, Display)>) + 'a
    ) -> Self {
        self.event_handler = Box::new(event_handler);
        self
    }
    pub fn with_settings(mut self, settings: Settings) -> Self {
        self.settings = settings;
        self
    }
    pub fn with_window_settings(mut self, window_settings: SimpleWindowBuilder) -> Self {
        self.window_settings = window_settings;
        self
    }
}
