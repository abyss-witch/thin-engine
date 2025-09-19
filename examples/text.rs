use thin_engine::{text_renderer::*, prelude::*};
use std::{cell::RefCell, rc::Rc};
fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    struct Graphics {
        shader: Program,
        indices: IndexBuffer<u32>,
        vertices: VertexBuffer<Vertex>,
        uvs: VertexBuffer<TextureCoords>
    }
    let graphics: Rc<RefCell<Option<Graphics>>> = Rc::default();
    let graphics_setup = graphics.clone();

    let mut font = Font::from_scale_and_file(40.0, "examples/DroidSans.ttf").unwrap();
    let draw_params = DrawParameters {
        blend: glium::Blend::alpha_blending(),
        backface_culling: glium::BackfaceCullingMode::CullingDisabled,
        ..params::alias_3d()
    };

    let time = std::time::Instant::now();

    let text = "Text can be drawn in 2d or 3d thanks to the power of Matrices. Text is drawn without wrapping and tab spacing, however the font struct has a function to format text for you.";
    thin_engine::builder(input_map!()).with_setup(|display, window, _| {
        window.set_title("Text Render");
        let (indices, vertices, uvs) = Font::mesh(display).unwrap();
        let shader = Font::shader(display).unwrap();
        graphics_setup.replace(Some(Graphics { indices, vertices, uvs, shader }));
    }).with_update(|_input, display, _settings, _target, window| {
        let graphics = graphics.borrow();
        let Graphics { shader, vertices, uvs, indices } = graphics.as_ref().unwrap();

        let text_renderer = TextRenderer {
            shader, indices, vertices, uvs, draw_params: &draw_params, display
        };

        let (width, height): (u32, u32) = window.inner_size().into();

        // set font size to 5% of the screens height
        let font_size = height as f32 * 0.05;
        font.resize(font_size);

        let perspective_2d = Mat4::perspective_2d((width, height)                  );
        let perspective_3d = Mat4::perspective_3d((width, height), 1.0, 0.1, 1024.0);

        let mut frame = display.draw();
        frame.clear_color_and_depth((0.9, 0.3, 0.5, 1.0), 11.0);

        let formated_text = font.format_text(text, Some(width as f32/font_size), 8, display);
        let formated_3d_text = font.format_text(text, Some(10.0), 8, display);
        let pos = vec3(-(width as f32 / height as f32), 1.0, 0.0);
        let time = time.elapsed().as_secs_f32();

        // 3d text
        text_renderer.draw(
            &formated_3d_text, vec3(1.0, 1.0, 0.5),
            &mut frame,
            Mat4::from_transform(
                vec3(0.0, 0.0, 5.0),
                Vec3::splat(0.3),
                Quat::from_y_rot(time*0.25) *
                Quat::from_z_rot(time*0.5 ) *
                Quat::from_x_rot(time     )
            ) * Mat4::from_pos(vec3(-5.0, 5.0, 0.0)),
            perspective_3d, Mat4::default(),
            &mut font
        ).unwrap();

        // 2d text
        text_renderer.draw(
            &formated_text, Vec3::ZERO, &mut frame,
            Mat4::from_pos_and_scale(pos, Vec3::splat(0.1)),
            perspective_2d, Mat4::default(), &mut font
        ).unwrap();

        frame.finish().unwrap();
    }).build(event_loop).unwrap()
}
