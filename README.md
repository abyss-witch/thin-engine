### Thin Engine
A thin game engine (hence the name) which ties some of my libraries (winit_input_map and glium-types)
into an easy to use, low boilerplate and high control game engine. Best suited for small projects.

Features:
    - Gamepad Support
    - Variable Input Support
    - Drawing through glium
    - Prebuilt shaders
    - Data Types for everything in glsl
    - Quaternions
    - Prebuilt meshes
```rust
use std::f32::consts::PI;
use thin_engine::{prelude::*, meshes::teapot};
use draw_parameters::BackfaceCullingMode;
#[derive(Hash, PartialEq, Eq, Clone, Copy)]
enum Action {
    Jump, Exit,
    Left, Right, Forward, Back,
    LookUp, LookDown, LookLeft, LookRight
}
use Action::*;

let (event_loop, window, display) = thin_engine::set_up().unwrap();
window.set_title("Walk Test");
let _ = window.set_cursor_grab(CursorGrabMode::Locked);
window.set_cursor_visible(false);

let input = {
    use thin_engine::input_map_setup::*;
    input_map!(
        (Jump,    Space,  GamepadButton::South),
        (Exit,    Escape, GamepadButton::Start),
        (Left,    ArrowLeft,  KeyA,  Axis(LeftStickX,  Neg)),
        (Right,   ArrowRight, KeyD,  Axis(LeftStickX,  Pos)),
        (Forward, ArrowUp,    KeyW,  Axis(LeftStickY,  Neg)),
        (Back,    ArrowDown,  KeyS,  Axis(LeftStickY,  Pos)),
        (LookRight, MouseMoveX(Pos), Axis(RightStickX, Pos)),
        (LookLeft,  MouseMoveX(Neg), Axis(RightStickX, Neg)),
        (LookUp,    MouseMoveY(Pos), Axis(RightStickY, Neg)),
        (LookDown,  MouseMoveY(Neg), Axis(RightStickY, Pos))
    )
};

let (indices, verts, norms) = mesh!(
    &display, &teapot::INDICES,
    &teapot::VERTICES, &teapot::NORMALS
);
let draw_parameters = DrawParameters {
    backface_culling: BackfaceCullingMode::CullClockwise,
    ..params::alias_3d()
};
let program = Program::from_source(
    &display, shaders::VERTEX,
    "#version 140
    out vec4 colour;
    in vec3 v_normal;
    uniform vec3 light;
    const vec3 albedo = vec3(0.1, 1.0, 0.3);
    void main(){
        float light_level = dot(light, v_normal);
        colour = vec4(albedo * light_level, 1.0);
    }", None,
).unwrap();

let mut pos = vec3(0.0, 0.0, -30.0);
let mut rot = vec2(0.0, 0.0);
let mut gravity = 0.0;

const DELTA: f32 = 0.016;
let settings = Settings::from_fps(60);
thin_engine::run(event_loop, input, settings, |input, _, target| {
    let size = window.inner_size().into();
    display.resize(size);
    let mut frame = display.draw();
    let view = Mat4::view_matrix_3d(size, 1.0, 1024.0, 0.1);

    //handle gravity and jump
    gravity += DELTA * 9.5;
    if input.pressed(Jump) {
        gravity = -10.0;
    }

    //set camera rotation
    let look_move = input.dir(LookRight, LookLeft, LookUp, LookDown);
    rot += look_move.scale(DELTA * 7.0);
    rot.y = rot.y.clamp(-PI / 2.0, PI / 2.0);
    let rx = Quaternion::from_y_rot(rot.x);
    let ry = Quaternion::from_x_rot(rot.y);
    let rot = rx * ry;

    //move player based on view and gravity
    let dir = input.dir_max_len_1(Right, Left, Forward, Back);
    let dir =  vec3(dir.x, 0.0, dir.y);
    pos += dir.transform(&rx.into()).scale(5.0*DELTA);
    pos.y = (pos.y - gravity * DELTA).max(0.0);

    if input.pressed(Exit) { target.exit() }

    frame.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
    //draw teapot
    frame.draw(
        (&verts, &norms), &indices,
        &program, &uniform! {
            view: view,
            model: Mat4::from_scale(Vec3::splat(0.1)),
            camera: Mat4::from_inverse_transform(pos, Vec3::ONE, rot),
            light: vec3(1.0, -0.9, -1.0).normalise()
        },
        &draw_parameters,
    ).unwrap();

    frame.finish().unwrap();
}).unwrap();
```
