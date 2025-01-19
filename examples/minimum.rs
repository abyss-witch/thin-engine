use thin_engine::prelude::*;
fn main() {
    let event_loop = EventLoop::new().unwrap();
    thin_engine::builder(input_map!()).with_update(|_, display, _, _, _| {
        let mut frame = display.draw();
        frame.clear_color(0.1, 0.7, 0.4, 1.0);
        frame.finish().unwrap();
    }).build(event_loop).unwrap();
}
