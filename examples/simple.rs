use std::{f32::consts::PI, rc::Rc, cell::RefCell};
use thin_engine::{prelude::*, meshes::teapot};
#[derive(Hash, PartialEq, Eq, Clone, Copy)]
enum Action {
    Jump, Exit,
    Left, Right, Forward, Back,
    LookUp, LookDown, LookLeft, LookRight
}
fn main() {
    use Action::*;
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    
    let input = { use base_input_codes::*; input_map!(
        (Jump,    Space,  GamepadInput::South),
        (Exit,    Escape, GamepadInput::Start),
        (Left,    ArrowLeft,  KeyA,  LeftStickLeft ),
        (Right,   ArrowRight, KeyD,  LeftStickRight),
        (Forward, ArrowUp,    KeyW,  LeftStickUp   ),
        (Back,    ArrowDown,  KeyS,  LeftStickDown ),
        (LookRight, MouseMoveRight, RightStickRight),
        (LookLeft,  MouseMoveLeft,  RightStickLeft ),
        (LookUp,    MouseMoveUp,    RightStickUp   ),
        (LookDown,  MouseMoveDown,  RightStickDown )
    )};

    struct Graphics {
        program: Program,
        indices: IndexBuffer<u16>,
        vertices: VertexBuffer<Vertex>,
        normals: VertexBuffer<Normal>
    }
    let graphics: Rc<RefCell<Option<Graphics>>> = Rc::new(RefCell::new(None));
    let graphics_setup = graphics.clone();
    
    let draw_parameters = DrawParameters {
        backface_culling: draw_parameters::BackfaceCullingMode::CullClockwise,
        ..params::alias_3d()
    };

    let mut pos = vec3(0.0, 0.0, -30.0);
    let mut rot = vec2(0.0, 0.0);
    let mut gravity = 0.0;

    let mut frame_start = Instant::now();

    thin_engine::builder(input).with_setup(|display, window, _| {
        let _ = window.set_cursor_grab(CursorGrabMode::Confined);
        let _ = window.set_cursor_grab(CursorGrabMode::Locked);
        window.set_cursor_visible(false);
        window.set_title("Walk Test");

        let (indices, vertices, normals) = mesh!(
            display, &teapot::INDICES, &teapot::VERTICES, &teapot::NORMALS
        );
        let program = Program::from_source(
            display, shaders::VERTEX,
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
        graphics_setup.replace(Some(Graphics { program, indices, vertices, normals }));
    }).with_update(|input, display, _, target, _| {
        let graphics = graphics.borrow();
        let Graphics { vertices, indices, normals, program } = graphics.as_ref().unwrap();
        let delta_time = frame_start.elapsed().as_secs_f32();
        frame_start = Instant::now();

        let mut frame = display.draw();
        let view = Mat4::view_matrix_3d(frame.get_dimensions(), 1.0, 1024.0, 0.1);

        //handle gravity and jump
        gravity += delta_time * 9.5;
        if input.pressed(Jump) { gravity = -10.0 }

        //set camera rotation
        let look_move = input.dir(LookLeft, LookRight, LookUp, LookDown);
        rot += look_move.scale(delta_time * 20.0);
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
            (vertices, normals), indices,
            program, &uniform! {
                view: view,
                model: Mat4::from_scale(Vec3::splat(0.1)),
                camera: Mat4::from_inverse_transform(pos, Vec3::ONE, rot),
                light: vec3(1.0, -0.9, -1.0).normalise()
            },
            &draw_parameters,
        ).unwrap();

        frame.finish().unwrap();
    }).build(event_loop).unwrap();
}
