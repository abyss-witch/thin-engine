use std::f32::consts::PI;
use thin_engine::{prelude::*, meshes::teapot};
#[derive(Hash, PartialEq, Eq, Clone, Copy)]
enum Action {
    Jump, Exit,
    Left, Right, Forward, Back,
    LookUp, LookDown, LookLeft, LookRight
}
fn main() {
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

    let mut frame_start = Instant::now();

    thin_engine::run(event_loop, input, Settings::from_fps(60), |input, _settings, target| {
        let delta_time = frame_start.elapsed().as_secs_f32();
        frame_start = Instant::now();

        display.resize(window.inner_size().into());
        let mut frame = display.draw();
        let view = Mat4::view_matrix_3d(frame.get_dimensions(), 1.0, 1024.0, 0.1);

        //handle gravity and jump
        gravity += delta_time * 9.5;
        if input.pressed(Jump) {
            gravity = -10.0;
        }

        //set camera rotation
        let look_move = input.dir(LookRight, LookLeft, LookUp, LookDown);
        rot += look_move.scale(delta_time * 7.0);
        rot.y = rot.y.clamp(-PI / 2.0, PI / 2.0);
        let rx = Quat::from_y_rot(rot.x);
        let ry = Quat::from_x_rot(rot.y);
        let rot = rx * ry;

        //move player based on view and gravity
        let dir = input.dir_max_len_1(Right, Left, Forward, Back);
        let move_dir = vec3(dir.x, 0.0, dir.y).scale(5.0*delta_time);
        pos += move_dir.transform(&Mat3::from_rot(rx));
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
}
