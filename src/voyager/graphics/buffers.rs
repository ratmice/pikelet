extern crate gl;
extern crate native;
extern crate cgmath;

use self::gl::types::*;
use std::mem;

use graphics::Bind;


pub struct VertexBuffer {
    id: GLuint
}

impl VertexBuffer {
    pub fn new(data: ~[f32], stride: u32) -> Option<VertexBuffer> {
        let buffer_length = (data.len() * mem::size_of::<GLfloat>()) as GLsizeiptr;
        let mut vbo = 0;

        unsafe {
            gl::GenBuffers(1, &mut vbo);
            if vbo > 0 {
                gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
                gl::BufferData(gl::ARRAY_BUFFER, buffer_length,
                               mem::transmute(&data[0]), gl::STATIC_DRAW);
            }
        }

        if vbo > 0 && gl::GetError() == gl::NO_ERROR {
            Some(VertexBuffer {
                id: vbo
            })
        } else {
            None
        }
    }
}

impl Bind for VertexBuffer {
    fn bind() {}
}

impl Drop for VertexBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}
