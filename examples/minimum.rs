use thin_engine::prelude::*;
fn main() {
    let (event_loop, _window, display) = thin_engine::set_up().unwrap();
    thin_engine::run(event_loop, input_map!(), Settings::default(), |_, _, _| {
        let mut frame = display.draw();
        frame.clear_color(0.1, 0.7, 0.4, 1.0);
        frame.finish().unwrap();
    }).unwrap();
}
