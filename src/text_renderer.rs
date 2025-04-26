//! a module for font renderering. 
//! ```
//! use thin_engine::{text_renderer::*, prelude::*};
//! use std::{cell::RefCell, rc::Rc};
//!     let event_loop = EventLoop::new().unwrap();
//!     event_loop.set_control_flow(ControlFlow::Poll);
//! 
//!     struct Graphics {
//!         shader: Program,
//!         indices: IndexBuffer<u32>,
//!         vertices: VertexBuffer<Vertex>,
//!         uvs: VertexBuffer<TextureCoords>
//!     }
//!     let graphics: Rc<RefCell<Option<Graphics>>> = Rc::default();
//!     let graphics_setup = graphics.clone();
//! 
//!     let mut font = Font::from_scale_and_file(40.0, "examples/DroidSans.ttf").unwrap();
//!     let draw_params = DrawParameters {
//!         blend: glium::Blend::alpha_blending(),
//!         backface_culling: glium::BackfaceCullingMode::CullingDisabled,
//!         ..params::alias_3d()
//!     };
//! 
//!     let time = std::time::Instant::now();
//! 
//!     let text = "Text can be drawn in 2d or 3d thanks to the power of Matrices. Text is drawn without wrapping and tab spacing however the font struct has a function to format text for you.";
//!     thin_engine::builder(input_map!()).with_setup(|display, _window, _event_loop| {
//!         let (indices, vertices, uvs) = Font::mesh(display);
//!         let shader = Font::shader(display).unwrap();
//!         graphics_setup.replace(Some(Graphics { indices, vertices, uvs, shader }));
//!     }).with_update(|_input, display, _settings, _target, window| {
//!         let graphics = graphics.borrow();
//!         let Graphics { shader, vertices, uvs, indices } = graphics.as_ref().unwrap();
//! 
//!         let text_renderer = TextRenderer {
//!             shader, indices, vertices, uvs, draw_params: &draw_params, display
//!         };
//! 
//!         let (width, height): (u32, u32) = window.inner_size().into();
//! 
//!         // set font size to 5% of the screens height
//!         let font_size = height as f32 * 0.05;
//!         font.resize(height as f32 * 0.05);
//! 
//!         let view_2d = Mat4::view_matrix_2d((width, height)                  );
//!         let view_3d = Mat4::view_matrix_3d((width, height), 1.0, 0.1, 1024.0);
//! 
//!         let mut frame = display.draw();
//!         frame.clear_color_and_depth((0.9, 0.3, 0.5, 1.0), 11.0);
//! 
//!         let formated_text = font.format_text(text, Some(width as f32/font_size), 8, display);
//!         let formated_3d_text = font.format_text(text, Some(10.0), 8, display);
//!         let pos = vec3(-(width as f32 / height as f32), 1.0, 0.0);
//!         let time = time.elapsed().as_secs_f32();
//! 
//!         // 3d text
//!         text_renderer.draw(
//!             &formated_3d_text, vec3(1.0, 1.0, 0.5),
//!             &mut frame,
//!             Mat4::from_transform(
//!                 vec3(0.0, 0.0, 5.0),
//!                 Vec3::splat(0.3),
//!                 Quat::from_y_rot(time*0.25) *
//!                 Quat::from_z_rot(time*0.5 ) *
//!                 Quat::from_x_rot(time     )
//!             ) * Mat4::from_pos(vec3(-5.0, 5.0, 0.0)),
//!             view_3d, Mat4::default(),
//!             &mut font
//!         ).unwrap();
//! 
//!         // 2d text
//!         text_renderer.draw(
//!             &formated_text, Vec3::ZERO, &mut frame,
//!             Mat4::from_pos_and_scale(pos, Vec3::splat(0.1)),
//!             view_2d, Mat4::default(), &mut font
//!         ).unwrap();
//! 
//!         frame.finish().unwrap();
//!     }).build(event_loop).unwrap();
//! ```
use crate::prelude::*;
use std::{fs::*, path::Path, borrow::Cow, ops::Deref, collections::HashMap};
use glium::{backend::Facade, texture::{RawImage2d, ClientFormat}, uniforms::SamplerWrapFunction};
pub use fontdue::{FontSettings, Metrics, OutlineBounds, LineMetrics};
/// a struct that represents a font and stores data for drawing. you can write your own renderer
/// using this struct but it is recommended to use the `TextRenderer` instead.
pub struct Font {
    scale: f32, font: fontdue::Font,
    textures: HashMap<char, (Metrics, Option<Texture2d>)>,
}
impl Font {
    /// mainly used for resizing when the window resolution is changed. if you want multiple
    /// drawing sizes, create multiple fonts with different scales
    pub fn resize(&mut self, scale: f32) {
        if (scale - self.scale).abs() <= f32::EPSILON { return }
        self.clear_loaded();
        self.scale = scale;
    }
    /// loads the mesh for use in the 'TextRenderer` struct
    pub fn mesh(display: &impl Facade) -> (IndexBuffer<u32>, VertexBuffer<Vertex>, VertexBuffer<TextureCoords>) {
        mesh!(display, &[1, 0, 2, 2, 3, 1], &[
            vec3(0.0, 1.0, 0.0).into(), vec3(0.0, 0.0, 0.0).into(),
            vec3(1.0, 1.0, 0.0).into(), vec3(1.0, 0.0, 0.0).into()
        ], &meshes::screen::UVS)
    }
    /// checks if the font has a glyph for the provided char
    pub fn has_glyph(&self, c: char) -> bool {
        self.font.has_glyph(c)
    }
    /// loads the shader for use in the `TextRenderer` struct.
    /// the fragmentshader has these uniforms, `albedo` is the colour of the text, `tex` is
    /// the texture of a glyph. also see the base vertex shader.
    pub fn shader(display: &impl Facade) -> Result<Program, glium::ProgramCreationError> {
        Program::from_source(display, shaders::VERTEX,
        "#version 140
        in vec2 uv;
        out vec4 colour;
        uniform vec3 albedo;
        uniform sampler2D tex;
        void main() {
            colour = vec4(albedo, texture(tex, uv).r);
        }", None)
    }
    /// creates a font with a scale measured in pixels
    pub fn from_scale_and_file(scale: f32, path: impl AsRef<Path>) -> Result<Self, &'static str> {
        let settings = FontSettings { scale, ..Default::default() };
        Self::from_settings_and_file(settings, path)
    }
    pub fn from_settings_and_file(settings: FontSettings, path: impl AsRef<Path>) -> Result<Self, &'static str> {
        let data = read(path).map_err(|_| "failed to open file")?;
        Self::from_settings_and_data(settings, data.as_slice())
    }
    pub fn from_settings_and_data(
        settings: FontSettings,
        data: impl Deref<Target = [u8]>
    ) -> Result<Self, &'static str> {
        let scale = settings.scale;
        let font = fontdue::Font::from_bytes(data, settings)?;
        Ok(Self {
            scale, font,
            textures: HashMap::new()
        })
    }
    /// gets character data for rendering **without** loading it to memmory. it is recommended
    /// to use `load_and_get` instead.
    pub fn char_data(&self, c: char, display: &impl Facade) -> (Metrics, Option<Texture2d>) {
        let (metrics, data) = self.font.rasterize(c, self.scale);
        if metrics.width == 0 { return (metrics, None) }
        let data = data.into_iter().rev().collect::<Vec<u8>>();
        let data = RawImage2d {
            data:   Cow::from(data),
            width:  metrics.width as u32,
            height: metrics.height as u32,
            format: ClientFormat::U8
        };
        (metrics, Some(Texture2d::new(display, data).unwrap()))
    }
    /// loads a characters texture and offset data to memmory for drawing unless the font file
    /// doesnt contatin said character
    pub fn load_char(&mut self, c: char, display: &impl Facade) {
        if self.font.has_glyph(c) {
            self.textures.insert(c, self.char_data(c, display));
        }
    }
    /// loads all characters that are valid in the font file to memory. (not recommended)
    pub fn load_all(&mut self, display: &impl Facade) {
        for c in self.font.chars().clone().into_keys() {
            self.load_char(c, display);
        }
    }
    /// tries to get texture and offset data. if the character is not in the font file
    /// returns replacement char data instead
    pub fn try_get(&self, mut c: char) -> Option<(Metrics, Option<&Texture2d>)> {
        if !self.font.has_glyph(c) { c = char::REPLACEMENT_CHARACTER }
        self.textures.get(&c).map(|(a, b)| (*a, b.as_ref()))
    }
    /// gets texture and offset data of a chararcter from memory. otherwise it
    pub fn load_and_get(&mut self, c: char, display: &impl Facade) -> (Metrics, Option<&Texture2d>) {
        let load = self.try_get(c).is_none();
        if load { self.load_char(c, display) }
        self.try_get(c).unwrap_or((Metrics {
            xmin: 0, ymin: 0, width: 0, height: 0,
            advance_width: 0.0, advance_height: 0.0,
            bounds: OutlineBounds { 
                xmin: 0.0, ymin: 0.0,
                width: 0.0, height: 0.0
            }
        }, None))
    }
    /// formats text so that tabs are replaced with `tab_indent` spaces and text is wrapped every
    /// time a words width exceeds `wrap`
    pub fn format_text(&mut self, text: &str, wrap: Option<f32>, tab_indent: usize, display: &impl Facade) -> String {
        let indent = " ".repeat(tab_indent);
        let text = text.replace('\t', &indent);
        if let Some(wrap) = wrap { self.wrap_text(&text, wrap, display) }
        else { text }
    }
    /// wraps text so that each word that excedes `wrap` in width is put on a new line.
    pub fn wrap_text(&mut self, text: &str, wrap: f32, display: &impl Facade) -> String {
        let wrap = wrap * self.scale;
        let mut lines = String::new();
        let mut width = 0.0;
        let mut word  = String::new();
        let mut word_width = 0.0;
        let mut space = String::new();
        
        for c in text.chars() {
            let (metrics, _) = self.load_and_get(c, display);
            let advance = if metrics.advance_width == 0.0 { metrics.advance_height }
                else { metrics.advance_width };

            let new_width = width + word_width + advance;
            let whitespace = c.is_whitespace();

            if c == '\n' {
                lines.push_str(&word);
                lines.push('\n');
                space.clear();
                word.clear();
                word_width = 0.0;
                width = 0.0;
            } else if new_width >= wrap && !whitespace {
                lines.push('\n');
                word.push(c);
                space.clear();
                width = 0.0;
            } else if whitespace {
                lines.push_str(&word);
                width = new_width;
                word_width = 0.0;
                word.clear();
                space.push(c);
            } else {
                lines.push_str(&space);
                space.clear();
                word.push(c);
                word_width += metrics.advance_width;
            }
        }
        lines.push_str(&word);
        lines
    }
    /// clears all loaded textures and offset data.
    pub fn clear_loaded(&mut self) { self.textures.clear() }
    /// metrics on line spacing for horizontal lines
    pub fn horizontal_metrics(&self) -> Option<LineMetrics> {
        self.font.horizontal_line_metrics(self.scale)
    }
    /// metrics on line spacing for vertical lines
    pub fn vertical_metrics(&self) -> Option<LineMetrics> {
        self.font.vertical_line_metrics(self.scale)
    }
}
pub struct TextRenderer<'a, F: Facade> {
    pub vertices:    &'a VertexBuffer<Vertex>,
    pub uvs:         &'a VertexBuffer<TextureCoords>,
    pub indices:     &'a IndexBuffer<u32>,
    pub shader:      &'a Program,
    pub draw_params: &'a DrawParameters<'a>,
    pub display:     &'a F
}
#[derive(Debug)]
pub enum TextDrawError {
    GliumDrawError(glium::DrawError),
    NoNewLineData
}
impl From<glium::DrawError> for TextDrawError {
    fn from(err: glium::DrawError) -> TextDrawError {
        Self::GliumDrawError(err)
    }
}
#[derive(Debug)]
pub enum DrawValidError {
    InvalidChar(char),
    DrawError(TextDrawError)
}
impl From<TextDrawError> for DrawValidError {
    fn from(err: TextDrawError) -> Self {
        Self::DrawError(err)
    }
}
impl<'a, F: Facade> TextRenderer<'a, F> {
    /// returns a `DrawValidError::InvalidChar(char)` when the text contains a character that the
    /// font doesnt have a glyph for instead of drawing the replacement char glyph
    pub fn try_draw_only_valid(
        &self, text: &str, colour: Vec3,
        frame: &mut impl Surface,
        model: Mat4, view: Mat4,
        camera: Mat4, font: &mut Font
    ) -> Result<(), DrawValidError> {
        if let Some(c) = text.chars().find(|&c| !font.has_glyph(c)) {
            return Err(DrawValidError::InvalidChar(c))
        }
        self.draw(text, colour, frame, model, view, camera, font)?;
        Ok(())
    }
    /// draws text. if drawing a character that is not in the font it will draw the replacement
    /// char instead. is this is undesirable try using `try_draw_only_valid` instead
    pub fn draw(
        &self, text: &str, colour: Vec3,
        frame: &mut impl Surface,
        model: Mat4, view: Mat4,
        camera: Mat4, font: &mut Font
    ) -> Result<(), TextDrawError> {
        let size = 1.0 / font.scale;

        let v_line = font.vertical_metrics();
        let h_line = font.horizontal_metrics();

        let (vertical, line_metric) = match (v_line, h_line) {
            (Some(v), Some(h)) => {
                let Some(c) = text.chars().next() else { return Ok(()) };
                let (metrics, _) = font.load_and_get(c, self.display);
                let vertical = metrics.advance_height == 0.0;
                (vertical, if vertical { v } else { h })
            },
            (Some(v), None) => (true,  v),
            (None, Some(h)) => (false, h),
            (None, None) => Err(TextDrawError::NoNewLineData)?
        };

        let mut pos = if vertical { vec2(-line_metric.ascent * size, 0.0) }
                             else { vec2(0.0, -line_metric.ascent * size) };

        for line in text.lines() {
            for c in line.chars() {
                let (metrics, tex) = font.load_and_get(c, self.display);
                let bounds = metrics.bounds;
                let offset = vec2(bounds.xmin * size, bounds.ymin * size);
                let draw_mat = Mat4::from_pos_and_scale(
                    (pos + offset).extend(0.0),
                    vec3(bounds.width*size, bounds.height*size, 1.0)
                );

                if let Some(tex) = tex { frame.draw(
                    (self.vertices, self.uvs), self.indices,
                    self.shader, &uniform! {
                        camera: camera, view: view,
                        model: model * draw_mat, albedo: colour,
                        tex: tex.sampled().wrap_function(SamplerWrapFunction::Clamp)
                    }, self.draw_params
                )? }

                if vertical { pos.y += metrics.advance_height * size }
                else        { pos.x += metrics.advance_width  * size }
            }
            if vertical { pos.y = 0.0; pos.x -= line_metric.new_line_size * size }
            else        { pos.x = 0.0; pos.y -= line_metric.new_line_size * size }
        }
        Ok(())
    }
}
