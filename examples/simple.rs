use std::{f32::consts::PI, thread, time::Duration};
use thin_engine::{glium_types::teapot, prelude::*};
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
    let mut input = Input::new([
        (vec![InputCode::keycode(KeyCode::Space)], Jump),
        (
            vec![
                InputCode::keycode(KeyCode::ArrowLeft),
                InputCode::keycode(KeyCode::KeyA),
            ],
            Left,
        ),
        (
            vec![
                InputCode::keycode(KeyCode::ArrowRight),
                InputCode::keycode(KeyCode::KeyD),
            ],
            Right,
        ),
        (
            vec![
                InputCode::keycode(KeyCode::ArrowUp),
                InputCode::keycode(KeyCode::KeyW),
            ],
            Forward,
        ),
        (
            vec![
                InputCode::keycode(KeyCode::ArrowDown),
                InputCode::keycode(KeyCode::KeyS),
            ],
            Back,
        ),
    ]);

    let (indices, verts, norms) = mesh!(
        &display,
        &teapot::INDICES,
        &teapot::VERTICES,
        &teapot::NORMALS
    );
    let draw_parameters = DrawParameters {
        backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
        ..params::alias_3d()
    };
    let program = Program::from_source(
        &display,
        shaders::VERTEX,
        "#version 140
        out vec4 colour;
        in vec3 v_normal;
        uniform vec3 light;
        void main(){
            colour = vec4(vec3(dot(light, normalize(v_normal))), 1.0);
        }",
        None,
    )
    .unwrap();

    let mut pos = vec3(0.0, 0.0, -10.0);
    let mut rot = vec2(0.0, 0.0);
    let mut gravity = 0.0;

    const DELTA: f32 = 0.016;

    thin_engine::run(
        event_loop,
        &mut input,
        |_, _| (),
        |input| {
            let mut frame = display.draw();
            let view = Mat4::view_matix(frame.get_dimensions(), 1.0, 1024.0, 0.1);

            //handle gravity and jump
            gravity += DELTA * 9.5;
            if input.pressed(Jump) {
                gravity = -10.0;
            }

            //set camera rotation
            rot += input.mouse_move.scale(DELTA);
            rot.y = rot.y.clamp(-PI / 2.0, PI / 2.0);
            let rx = Quaternion::from_y_rotation(rot.x);
            let ry = Quaternion::from_x_rotation(rot.y);
            let rot = rx * ry;

            //move player based on view and gravity
            let move_dir =
                vec3(input.axis(Right, Left), 0.0, input.axis(Forward, Back)).normalise();
            pos += move_dir.transform(&Mat3::from_rot(rx)).scale(5.0 * DELTA);
            pos -= vec3(0.0, gravity, 0.0).scale(DELTA);

            frame.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
            //draw teapot
            frame
                .draw(
                    (&verts, &norms),
                    &indices,
                    &program,
                    &uniform! {
                        view: view,
                        model: Mat4::from_scale(vec3(0.1, 0.1, 0.1)),
                        camera: Mat4::from_pos_and_rot(pos, rot),
                        light: vec3(1.0, -0.9, -1.0).normalise()
                    },
                    &draw_parameters,
                )
                .unwrap();

            frame.finish().unwrap();
            thread::sleep(Duration::from_millis(16));
        },
    )
    .unwrap();
}
