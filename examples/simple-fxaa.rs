use std::f32::consts::PI;
use thin_engine::{
    ResizableTexture2D, ResizableDepthTexture2D,
    prelude::*, glium::framebuffer::*,
    meshes::{teapot, screen}
};
#[derive(Hash, PartialEq, Eq, Clone, Copy)]
enum Action {
    Left, Right, Forward, Back,
    LookLeft, LookRight, LookUp, LookDown,
    FXAA
}
fn main() {
    use Action::*;
    let (event_loop, window, display) = thin_engine::set_up().unwrap();
    window.set_title("FXAA Test");
    let _ = window.set_cursor_grab(CursorGrabMode::Locked);
    window.set_cursor_visible(false);

    let mut colour = ResizableTexture2D::default();
    let mut depth = ResizableDepthTexture2D::default();

    
    
    let input = {
        use thin_engine::input_map_setup::*;
        input_map!(
            (Left,      ArrowLeft, KeyA, Axis(LeftStickX,  Neg)),
            (Right,     ArrowRight,KeyD, Axis(LeftStickX,  Pos)),
            (Forward,   ArrowUp,   KeyW, Axis(LeftStickY,  Neg)),
            (Back,      ArrowDown, KeyS, Axis(LeftStickY,  Pos)),
            (LookRight, MouseMoveX(Pos), Axis(RightStickX, Pos)),
            (LookLeft,  MouseMoveX(Neg), Axis(RightStickX, Neg)),
            (LookUp,    MouseMoveY(Pos), Axis(RightStickY, Pos)),
            (LookDown,  MouseMoveY(Neg), Axis(RightStickY, Neg)),
            (FXAA,      KeyF,            GamepadButton::North  )
        )
    };
    let (screen_indices, verts, uvs) = mesh!(
        &display, &screen::INDICES, &screen::VERTICES, &screen::UVS
    );
    let screen_mesh = (&verts, &uvs);
    let (indices, verts, norms) = mesh!(
        &display, &teapot::INDICES, &teapot::VERTICES, &teapot::NORMALS
    );
    let teapot_mesh = (&verts, &norms);
    let draw_parameters = DrawParameters {
        backface_culling: draw_parameters::BackfaceCullingMode::CullClockwise,
        ..params::alias_3d()
    };
    let mut fxaa_on = true;
    let program = Program::from_source(
        &display, shaders::VERTEX,
        "#version 140
        out vec4 colour;
        in vec3 v_normal;
        uniform vec3 light;
        uniform mat4 camera;
        uniform vec3 ambient;
        uniform vec3 albedo;
        uniform float shine;
        void main() {
            vec3 camera_dir = inverse(mat3(camera)) * vec3(0, 0, -1);
            vec3 half_dir = normalize(camera_dir + light);
            float specular = pow(max(dot(half_dir, v_normal), 0.0), shine);
            float light_level = max(dot(light, v_normal), 0.0);
            colour = vec4(albedo * light_level + ambient + vec3(specular), 1.0);
        }", None
    ).unwrap();
    let normal = Program::from_source(
        &display, shaders::SCREEN_VERTEX, 
        "#version 140
        in vec2 uv;
        uniform sampler2D tex;
        out vec4 colour;
        void main() {
            colour = texture(tex, uv);
        }", None
    ).unwrap();
    let fxaa = shaders::fxaa_shader(&display).unwrap();
    
    let mut pos = vec3(0.0, 0.0, -30.0);
    let mut rot = vec2(0.0, 0.0);
    const DELTA: f32 = 0.016;

    thin_engine::run(event_loop, input, Settings::from_fps(60), |input, _, _| {
        // using a small resolution to show the effect.
        // `let size = window.inner_size().into();` 
        // can be used instead to set resolution to window size
        let size = (380, 216);
        display.resize(size);
        depth.resize_to_display(&display);
        colour.resize_to_display(&display);

        //press f to toggle FXAA
        if input.pressed(FXAA) { fxaa_on = !fxaa_on }

        let colour = colour.texture.as_ref().unwrap();
        let depth = depth.texture.as_ref().unwrap();
        let mut frame = SimpleFrameBuffer::with_depth_buffer(
            &display, colour, depth
        ).unwrap();

        let view = Mat4::view_matrix_3d(size, 1.0, 1024.0, 0.1);

        //set camera rotation
        let look_move = input.dir(LookRight, LookLeft, LookUp, LookDown);
        rot += look_move.scale(DELTA * 2.0);
        rot.y = rot.y.clamp(-PI / 2.0, PI / 2.0);
        let rx = Quat::from_y_rot(rot.x);
        let ry = Quat::from_x_rot(rot.y);
        let rot = rx * ry;

        //move player based on view
        let dir = input.dir_max_len_1(Right, Left, Forward, Back);
        let move_dir = vec3(dir.x, 0.0, dir.y).scale(5.0*DELTA);
        pos += move_dir.transform(&Mat3::from_rot(rx));

        frame.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
        //draw teapot
        frame.draw(
            teapot_mesh, &indices,
            &program, &uniform! {
                view: view,
                model: Mat4::from_scale(Vec3::splat(0.1)),
                camera: Mat4::from_inverse_transform(pos, Vec3::ONE, rot),
                light: vec3(0.1, 0.25, -1.0).normalise(),
                albedo: vec3(0.5, 0.1, 0.4),
                ambient: vec3(0.0, 0.05, 0.1),
                shine: 50.0f32,
            },
            &draw_parameters,
        ).unwrap();

        let mut frame = display.draw();
        frame.draw(
            screen_mesh, &screen_indices, if fxaa_on { &fxaa } else { &normal },
            &shaders::fxaa_uniforms(colour), &DrawParameters::default()
        ).unwrap();
        frame.finish().unwrap();
        thread::sleep(Duration::from_millis(16));
    }).unwrap();
}
