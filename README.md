A thin engine between glium, winit, glium-types and winit-input-map with some extra helper functions and shaders. Easy fxaa support.
```rust
use std::{f32::consts::PI, thread, time::Duration};
use thin_engine::{prelude::*, meshes::teapot};
enum Action {
    Jump,
    Left,
    Right,
    Forward,
    Back,
}
impl Into<usize> for Action {
    fn into(self) -> usize {
        self as usize
    }
}
fn main() {
    use Action::*;
    let (event_loop, window, display) = thin_engine::set_up().unwrap();
    window.set_title("Walk Test");
    let _ = window.set_cursor_grab(CursorGrabMode::Locked);
    window.set_cursor_visible(false);

    let mut input = InputMap::new([
        (vec![Input::keycode(KeyCode::Space)], Jump),
        (vec![Input::keycode(KeyCode::ArrowLeft), Input::keycode(KeyCode::KeyA)], Left),
        (vec![Input::keycode(KeyCode::ArrowRight), Input::keycode(KeyCode::KeyD)], Right),
        (vec![Input::keycode(KeyCode::ArrowUp), Input::keycode(KeyCode::KeyW)], Forward),
        (vec![Input::keycode(KeyCode::ArrowDown), Input::keycode(KeyCode::KeyS)], Back)
    ]);

    let (indices, verts, norms) = mesh!(
        &display, &teapot::INDICES, &teapot::VERTICES, &teapot::NORMALS
    );
    let draw_parameters = DrawParameters {
        backface_culling: draw_parameters::BackfaceCullingMode::CullClockwise,
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

    thin_engine::run(event_loop, &mut input, |input| {
        display.resize(window.inner_size().into());
        let mut frame = display.draw();
        let view = Mat4::view_matrix_3d(frame.get_dimensions(), 1.0, 1024.0, 0.1);

        //handle gravity and jump
        gravity += DELTA * 9.5;
        if input.pressed(Jump) {
            gravity = -10.0;
        }

        //set camera rotation
        rot += input.mouse_move.scale(DELTA * 2.0);
        rot.y = rot.y.clamp(-PI / 2.0, PI / 2.0);
        let rx = Quaternion::from_y_rotation(rot.x);
        let ry = Quaternion::from_x_rotation(rot.y);
        let rot = rx * ry;

        //move player based on view and gravity
        let x = input.axis(Right, Left);
        let y = input.axis(Forward, Back);
        let move_dir = vec3(x, 0.0, y).normalise();
        pos += move_dir.transform(&Mat3::from_rot(rx)).scale(5.0 * DELTA);
        pos.y = (pos.y - gravity * DELTA).max(0.0);

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
        thread::sleep(Duration::from_millis(16));
    }).unwrap();
}
```
