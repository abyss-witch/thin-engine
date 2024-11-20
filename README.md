### Thin Engine
A thin game engine (hence the name) which ties some of my libraries (winit-input-map and glium-types)
into an easy to use, low boilerplate and high control game engine. Best suited for small projects.

Features:
    - Gamepad Support
    - Variable Input Support
    - Drawing through glium
    - Prebuilt shaders
    - Data Types for everything in glsl
    - Quaternions
    - Prebuilt meshes
    - optional text renderer
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
window.set_cursor_grab(CursorGrabMode::Confined).ok();
window.set_cursor_grab(CursorGrabMode::Locked).ok();
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
let program = Progronam::from_source(
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

let mut frame_start = Instant::now();
let settings = Settings::from_fps(60);
thin_engine::run(event_loop, input, settings, |input, _, target| {
    let delta_time = frame_start.elapsed().as_secs_f32();
    frame_start = Instant::now();

    let size = window.inner_size().into();
    display.resize(size);
    let mut frame = display.draw();
    let view = Mat4::view_matrix_3d(size, 1.0, 1024.0, 0.1);

    //handle gravity and jump
    gravity += delta_time * 9.5;
    if input.pressed(Jump) {
        gravity = -10.0;
    }

    //set camera rotation
    let look_move = input.dir(LookRight, LookLeft, LookUp, LookDown);
    rot += look_move.scale(delta_time * 7.0);
    rot.y = rot.y.clamp(-PI / 2.0, PI / 2.0);
    let rx = Quaternion::from_y_rot(rot.x);
    let ry = Quaternion::from_x_rot(rot.y);
    let rot = rx * ry;

    //move player based on view and gravity
    let dir = input.dir_max_len_1(Right, Left, Forward, Back);
    let dir =  vec3(dir.x, 0.0, dir.y);
    pos += dir.transform(&rx.into()).scale(5.0*delta_time);
    pos.y = (pos.y - gravity * delta_time).max(0.0);

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

### Recomended Drawing Abstraction
It is encouraged to make structs for abstracting drawing that is used often. A good way to structure the data is like so:
```rust
pub struct Drawer<'a> {
    pub display: &'a thin_engine::Display,
    pub vertices: &'a VertexBuffer<Vertex>,
    pub indices: &'a IndexBuffer<u32>,
    pub shader: &'a Program,
}
impl Drawerer {
    pub fn draw(&self, &mut impl glium::Surface) -> Result<(), glium::DrawError> {
        // your drawing code
    }
}
```
and initialising the data like this:
```rust
let drawerer = Drawer {
    pub display: &display,
    pub vertices: &vertices,
    pub indices: &indices,
    pub shader: &shader
}
```
this allows for reuse of the display and even shaders and meshes while still removing boilerplate

### Optional Text Renderer
If you want to use the text renderer enable the text feature by adding this to your Cargo.toml
```toml
[dependencies]
thin-engine = { version = "*", features = ["text"] }
```
The text renderer can be setup like this
```rust
use thin_engine::{prelude::*, text_renderer::*};

let (indices, vertices, uvs) = Font::mesh(&display);
let text_shader = Font::shader(&display).unwrap();
let draw_params = DrawParameters {
    blend: glium::Blend::alpha_blending(),
    ..Default::default()
};

let mut font = Font::from_scale_and_file(40.0, "file/location").unwrap();
let text_renderer = TextRenderer {
    shader: &text_shaderm indices: &indices, vertices: &vertices,
    uv: &uvs, draw_params: &draw_params, display: &display
};
```
and then can be drawn in the loop like so
```rust
// draw the sentence Hello World in green text
text_renderer.draw(
    "Hello World", vec3(0.0, 1.0, 0.0), &mut frame,
    model_mat, view_mat, camera_mat, &mut font
).unwrap();
```
for wrapping and tab spacing of text you can use
```rust
font.format_text(text, Some(wrap), tab_indent, &display);
```
