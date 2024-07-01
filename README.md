A thin game engine (hence the name). Drawing done with `glium`, game variables done with
`glium-types`, windowing done with `winit` and input done with `winit-input-map`. It has easy fxaa
support and low boilerplate despite having lots of control.
```
use thin_engine::{prelude::*, meshes::screen};
use Action::*;

#[derive(ToUsize)]
enum Action {
    Left,
    Right,
    Jump,
    Exit
}
let (event_loop, window, display) = thin_engine::set_up().unwrap();
let mut input = input_map!(
    (Left,  KeyCode::KeyA, MouseButton::Left, KeyCode::ArrowLeft),
    (Right, KeyCode::KeyD, MouseButton::Right, KeyCode::ArrowRight),
    (Jump,  KeyCode::KeyW, KeyCode::ArrowUp, KeyCode::Space),
    (Exit,  KeyCode::Escape)
);
let (box_indices, box_verts) = mesh!(
    &display, &screen::INDICES, &screen::VERTICES
);
let box_shader = Program::from_source(
&display, shaders::VERTEX, 
"#version 140
out vec4 colour;
void main() {
    colour = vec4(1.0, 0.0, 0.0, 1.0);
}", None).unwrap();

let mut player_pos = Vec2::ZERO;
let mut player_gravity = 0.0;
let mut player_can_jump = true;
// camera matrix must be inverse
let camera = Mat4::from_scale(Vec3::splat(10.0)).inverse();

// target of 60 fps
let target_delay = Duration::from_secs_f32(1.0/60.0);
let mut delta_time = 0.016; // change in time between frames

thin_engine::run(event_loop, &mut input, |input, target| {
    let frame_start = Instant::now();
    // set up frame
    let size = window.inner_size().into();
    display.resize(size);
    let mut frame = display.draw();
    let view2d = Mat4::view_matrix_2d(size);
    
    // game logic
    player_pos.x += input.axis(Right, Left) * 10.0 * delta_time;
    player_gravity += delta_time * 50.0;
    player_pos.y -= player_gravity * delta_time;
    if player_pos.y < 0.0 {
        player_pos.y = 0.0;
        player_can_jump = true;
    }
    if player_can_jump && input.pressed(Jump) {
        player_gravity = -20.0;
        player_can_jump = false;
    }
    // draw
    frame.clear_color(0.0, 0.0, 0.0, 1.0);
    frame.draw(
        &box_verts, &box_indices, &box_shader, &uniform! {
            view: view2d, camera: camera,
            model: Mat4::from_pos(player_pos.extend(0.0)),
        }, &DrawParameters::default()
    );
    
    frame.finish().unwrap();
    if input.pressed(Exit) { target.exit() }
    thread::sleep(target_delay.saturating_sub(frame_start.elapse()));
    delta_time = frame_start.elapsed().as_secs_f32();
}).unwrap();
```
