use glium::{uniform, uniforms::*, Program, program::ProgramCreationError, texture::*};
use crate::Display;
use glium_types::prelude::*;
pub use glium_types::shaders::VERTEX;

/// vertex shader that doesnt change position or uvs. takes in `Vertex` and `TextureCoords`
/// and outputs `uv`. intended for use with the screen mesh to draw over
/// the screen. e.g. with the fxaa shader.
pub const SCREEN_VERTEX: &str =
"#version 140
in vec3 position;
in vec2 texture_coords;
out vec2 uv;
void main() {
    uv = texture_coords;
    gl_Position = vec4(position, 1);
}";
/// fragment shader to smooth the harsh edges of a render.
/// see `fxaa_shader()`
pub const FXAA: &str = 
"/**
For working in thin engine some variables have been changed, same as formatting.
--
Basic FXAA implementation based on the code on geeks3d.com with the
modification that the texture2DLod stuff was removed since it's
unsupported by WebGL.
--

From:
https://github.com/mattdesl/glsl-fxaa
https://github.com/mitsuhiko/webgl-meincraft

Copyright (c) 2011 by Armin Ronacher.

Some rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are
met:

    * Redistributions of source code must retain the above copyright
      notice, this list of conditions and the following disclaimer.

    * Redistributions in binary form must reproduce the above
      copyright notice, this list of conditions and the following
      disclaimer in the documentation and/or other materials provided
      with the distribution.

    * The names of the contributors may not be used to endorse or
      promote products derived from this software without specific
      prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
\"AS IS\" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/
#version 140

const float FXAA_REDUCE_MIN = 1.0 / 128.0;
const float FXAA_REDUCE_MUL = 1.0 / 8.0;
const float FXAA_SPAN_MAX = 8.0;

in vec2 uv;
out vec4 colour;

uniform sampler2D tex;
uniform vec2 pixel_size;

//optimized version for mobile, where dependent 
//texture reads can be a bottleneck
void main() {
    vec3 rgbNW = texture(tex, uv + pixel_size*0.5).xyz;
    vec3 rgbNE = texture(tex, uv + vec2(-pixel_size.x, pixel_size.y)*0.5).xyz;
    vec3 rgbSW = texture(tex, uv + vec2(pixel_size.x, -pixel_size.y)*0.5).xyz;
    vec3 rgbSE = texture(tex, uv - pixel_size*0.5).xyz;
    vec4 texColor = texture(tex, uv);
    vec3 rgbM  = texColor.xyz;
    vec3 luma = vec3(0.299, 0.587, 0.114);
    float lumaNW = dot(rgbNW, luma);
    float lumaNE = dot(rgbNE, luma);
    float lumaSW = dot(rgbSW, luma);
    float lumaSE = dot(rgbSE, luma);
    float lumaM  = dot(rgbM,  luma);
    float lumaMin = min(lumaM, min(min(lumaNW, lumaNE), min(lumaSW, lumaSE)));
    float lumaMax = max(lumaM, max(max(lumaNW, lumaNE), max(lumaSW, lumaSE)));
    
    vec2 dir = vec2(
        -((lumaNW + lumaNE) - (lumaSW + lumaSE)),
        ((lumaNW + lumaSW) - (lumaNE + lumaSE))
    );
    
    float dirReduce = max(
        (lumaNW + lumaNE + lumaSW + lumaSE)*0.25*FXAA_REDUCE_MUL,
        FXAA_REDUCE_MIN
    );
    
    float rcpDirMin = 1.0 / (min(abs(dir.x), abs(dir.y)) + dirReduce);
    dir = min(
        vec2(FXAA_SPAN_MAX, FXAA_SPAN_MAX),
        max(-vec2(FXAA_SPAN_MAX), dir*rcpDirMin)
    ) * pixel_size;
    
    vec3 rgbA = 0.5 * (
        texture(tex, uv + dir * (1.0 / 3.0 - 0.5)).xyz +
        texture(tex, uv + dir * (2.0 / 3.0 - 0.5)).xyz
    );
    vec3 rgbB = rgbA * 0.5 + 0.25 * (
        texture(tex, uv + dir * -0.5).xyz +
        texture(tex, uv + dir * 0.5).xyz
    );

    float lumaB = dot(rgbB, luma);
    if ((lumaB < lumaMin) || (lumaB > lumaMax)) {
        colour = vec4(rgbA, texColor.a);
    } else {
        colour = vec4(rgbB, texColor.a);
    }
}";
/// a shader for smoothing jaggerd pixel edges. use with `fxaa_uniforms` with the input of a
/// texture. (check simple-fxaa example)
pub fn fxaa_shader(display: &Display) -> Result<Program, ProgramCreationError> {
    Program::from_source(display, SCREEN_VERTEX, FXAA, None)
}
/// for use with the fxaa shader.
pub fn fxaa_uniforms(tex: &Texture2d) -> UniformsStorage<'_, Vec2, UniformsStorage<'_, Sampler<'_, Texture2d>, EmptyUniforms>>{
    uniform! {
        tex: Sampler::new(tex),
        pixel_size: Vec2::ONE / vec2(tex.width() as f32, tex.height() as f32)
    }
}
