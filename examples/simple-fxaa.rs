use std::{f32::consts::PI, rc::Rc, cell::RefCell};
use thin_engine::{
    ResizableTexture2d, ResizableDepthTexture2d,
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
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut colour = ResizableTexture2d::default();
    let mut depth = ResizableDepthTexture2d::default();

    let input = { use base_input_codes::*; input_map!(
        (Left,    ArrowLeft,  KeyA, LeftStickLeft ),
        (Right,   ArrowRight, KeyD, LeftStickRight),
        (Forward, ArrowUp,    KeyW, LeftStickUp   ),
        (Back,    ArrowDown,  KeyS, LeftStickDown ),
        (LookRight, MouseMoveRight, RightStickRight),
        (LookLeft,  MouseMoveLeft,  RightStickLeft ),
        (LookUp,    MouseMoveUp,    RightStickUp   ),
        (LookDown,  MouseMoveDown,  RightStickDown ),
        (FXAA,      KeyF,       GamepadInput::North)
    ) };
    struct Graphics {
        screen_indices: IndexBuffer<u32>,
        screen_vertices: VertexBuffer<Vertex>,
        screen_uvs: VertexBuffer<TextureCoords>,

        teapot_indices: IndexBuffer<u16>,
        teapot_vertices: VertexBuffer<Vertex>,
        teapot_uvs: VertexBuffer<TextureCoords>,
        teapot_normals: VertexBuffer<Normal>,

        fxaa: Program, normal: Program, program: Program
    }
    let graphics: Rc<RefCell<Option<Graphics>>> = Rc::default();
    let graphics_setup = graphics.clone();

    let draw_parameters = DrawParameters {
        backface_culling: draw_parameters::BackfaceCullingMode::CullClockwise,
        ..params::alias_3d()
    };
    let mut fxaa_on = true;
    
    let mut pos = vec3(0.0, 0.0, -30.0);
    let mut rot = vec2(0.0, 0.0);
    
    let mut frame_start = Instant::now();

    thin_engine::builder(input).with_setup(|display, window, _| {
        window.set_title("FXAA Test");
        let _ = window.set_cursor_grab(CursorGrabMode::Confined);
        let _ = window.set_cursor_grab(CursorGrabMode::Locked);
        window.set_cursor_visible(false);

        let (screen_indices, screen_vertices, screen_uvs) = mesh!(
            display, &screen::INDICES, &screen::VERTICES, &screen::UVS
        );
        let (teapot_indices, teapot_vertices, teapot_uvs, teapot_normals) = mesh!(
            display, &teapot::INDICES, &teapot::VERTICES, &[] as &[TextureCoords; 0], &teapot::NORMALS
        );

        let program = Program::from_source(
            display, shaders::VERTEX,
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
        let fxaa = shaders::fxaa_shader(display).unwrap();
        let normal = Program::from_source(
            display, shaders::SCREEN_VERTEX, 
            "#version 140
            in vec2 uv;
            uniform sampler2D tex;
            out vec4 colour;
            void main() {
                colour = texture(tex, uv);
            }", None
        ).unwrap();
        graphics_setup.replace(Some(Graphics {
            screen_indices, screen_vertices, screen_uvs,
            teapot_indices, teapot_vertices, teapot_uvs, teapot_normals,
            program, normal, fxaa
        }));
    }).with_update(|input, display, _, _, _| {
        let graphics = graphics.borrow();
        let Graphics {
            screen_indices, screen_vertices, screen_uvs,
            teapot_indices, teapot_vertices, teapot_uvs, teapot_normals,
            program, normal, fxaa
        } = graphics.as_ref().unwrap();
        let teapot_mesh = (teapot_vertices, teapot_normals, teapot_uvs);
        let screen_mesh = (screen_vertices, screen_uvs);

        let delta_time = frame_start.elapsed().as_secs_f32();
        frame_start = Instant::now();

        // using a small resolution to better show the effect of fxaa.
        let size = (380, 216);
        display.resize(size);
        depth.resize_to_display(&display);
        colour.resize_to_display(&display);

        // press f to toggle FXAA
        if input.pressed(FXAA) { fxaa_on = !fxaa_on }

        let colour = colour.texture();
        let depth = depth.texture();
        let mut frame = SimpleFrameBuffer::with_depth_buffer(
            display, colour, depth
        ).unwrap();

        let view = Mat4::view_matrix_3d(size, 1.0, 1024.0, 0.1);

        // set camera rotation
        let look_move = input.dir(LookLeft, LookRight, LookUp, LookDown);
        rot += look_move.scale(delta_time * 20.0);
        rot.y = rot.y.clamp(-PI / 2.0, PI / 2.0);
        let rx = Quat::from_y_rot(rot.x);
        let ry = Quat::from_x_rot(rot.y);
        let rot = rx * ry;

        // move player based on view
        let dir = input.dir_max_len_1(Right, Left, Forward, Back);
        let move_dir = vec3(dir.x, 0.0, dir.y).scale(5.0*delta_time);
        pos += move_dir.transform(&Mat3::from_rot(rx));

        frame.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
        // draw teapot
        frame.draw(
            teapot_mesh, teapot_indices,
            program, &uniform! {
                view: view,
                model: Mat4::from_scale(Vec3::splat(0.1)),
                camera: Mat4::from_inverse_transform(pos, Vec3::ONE, rot),
                light:   vec3(0.1, 0.25, -1.0).normalise(),
                albedo:  vec3(0.5, 0.1,   0.4),
                ambient: vec3(0.0, 0.05,  0.1),
                shine: 50.0f32,
            },
            &draw_parameters,
        ).unwrap();

        let mut frame = display.draw();
        frame.draw(
            screen_mesh, screen_indices, if fxaa_on { fxaa } else { normal },
            &shaders::fxaa_uniforms(colour), &DrawParameters::default()
        ).unwrap();
        frame.finish().unwrap();
    }).build(event_loop).unwrap();
}
