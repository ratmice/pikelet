// Copyright Brendan Zabarauskas 2014

use gfx;
use std::fmt;

#[vertex_format]
pub struct Vertex {
    #[name = "a_Pos"]
    pub pos: [f32, ..3],

    #[name = "a_Color"]
    pub color: [f32, ..3],
}

impl Clone for Vertex {
    fn clone(&self) -> Vertex {
        Vertex { pos: self.pos, color: self.color }
    }
}

impl fmt::Show for Vertex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ {}, {}, {} }}", self.pos[0], self.pos[1], self.pos[2])
    }
}

#[shader_param(Batch)]
pub struct Params {
    #[name = "u_Transform"]
    pub transform: [[f32, ..4], ..4],
}

pub static VERTEX_SRC: gfx::ShaderSource<'static> = shaders! {
GLSL_120: b"
    #version 120

    attribute vec3 a_Pos;
    attribute vec3 a_Color;
    varying vec3 v_Color;

    uniform mat4 u_Transform;

    void main() {
        v_Color = a_Color;
        gl_Position = u_Transform * vec4(a_Pos, 1.0);
    }
"
GLSL_150: b"
    #version 150 core

    in vec3 a_Pos;
    in vec3 a_Color;
    out vec3 v_Color;

    uniform mat4 u_Transform;

    void main() {
        v_Color = a_Color;
        gl_Position = u_Transform * vec4(a_Pos, 1.0);
    }
"
};

pub static FRAGMENT_SRC: gfx::ShaderSource<'static> = shaders! {
GLSL_120: b"
    #version 120

    varying vec3 v_Color;
    out vec4 o_Color;

    void main() {
        o_Color = vec4(v_Color, 1.0);
    }
"
GLSL_150: b"
    #version 150 core

    in vec3 v_Color;
    out vec4 o_Color;

    void main() {
        o_Color = vec4(v_Color, 1.0);
    }
"
};
