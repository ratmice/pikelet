extern crate gl;

use self::gl::types::*;
use std::u16;
use std::vec::Vec;

use platform::Platform;

trait GLBuffer {
    fn gen() -> GLuint;
    fn delete();
    fn bind();
    fn data<T>(buf: Vec<T>);
}

struct IndexBuffer {
    id: GLuint
}

struct VertexBuffer {
    id: GLuint
}

struct UniformBuffer {
    id: GLuint
}

struct ShaderProgram {
    vertex_shader: GLuint,
    fragment_shader: GLuint,
    geometry_shader: Option<GLuint>,
    tesselation_shader: Option<GLuint>,
    compute_shader: Option<GLuint>
}

#[deriving(Clone, Eq)]
pub struct Handle {
    ref_index: u16,
    generation: u16
}

/// A graphics device manager
pub struct GraphicsManager {
    index_buffers: Vec<IndexBuffer>,
    vertex_buffers: Vec<VertexBuffer>,
    uniform_buffers: Vec<UniformBuffer>,
    shader_programs: Vec<ShaderProgram>,
}

impl GraphicsManager {
    /// Initialise a new graphics device manager
    pub fn init<T: Platform>(platform: &T) -> GraphicsManager {
        platform.load_gl(gl::load_with);
        
        gl::ClearColor(0.3,0.3,0.3,1.0);
        
        GraphicsManager {
            index_buffers: Vec::with_capacity(u16::MAX as uint),
            vertex_buffers: Vec::with_capacity(u16::MAX as uint),
            uniform_buffers: Vec::with_capacity(u16::MAX as uint),
            shader_programs: Vec::with_capacity(u16::MAX as uint),
        }
    }

    pub fn clear(&self) {
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }

    pub fn destroy(self) {}

    pub fn add_vertex_buffer<T>(&mut self, _data: ~[T], _stride: u32) -> Handle {
        fail!("Not yet implemented.");
    }

    pub fn destroy_vertex_buffer(&mut self, _handle: Handle) {
        fail!("Not yet implemented.");
    }

    pub fn add_index_buffer(&mut self, _data: ~[u32]) -> Handle {
        fail!("Not yet implemented.");
    }

    pub fn destroy_index_buffer(&mut self, _handle: Handle) {
        fail!("Not yet implemented.");
    }

    pub fn add_uniform_buffer(&mut self/*, ...*/) -> Handle {
        fail!("Not yet implemented.");
    }

    pub fn destroy_uniform_buffer(&mut self, _handle: Handle) {
        fail!("Not yet implemented.");
    }

    pub fn add_shader_program(&mut self, _shaders: ~[()/*Shader*/]) -> Handle {
        fail!("Not yet implemented.");
    }

    pub fn destroy_shader_program(&mut self, _handle: Handle) {
        fail!("Not yet implemented.");
    }
}

impl Drop for GraphicsManager {
    fn drop(&mut self) {
        // Clean up all the things
    }
}
