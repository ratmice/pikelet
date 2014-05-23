extern crate gl;
extern crate native;
extern crate cgmath;

use platform::Platform;

use super::data::Handle;
use super::data::EngineData;

use self::shader_program::ShaderProgram;
use self::buffers::VertexBuffer;

mod shader_program;
mod buffers;

trait GLObject {
    fn bind();
}

pub type ShaderProgramHandle = Handle;
pub type VertexBufferHandle = Handle;

/// A graphics device manager
pub struct GraphicsManager {
    vertex_buffers: EngineData<VertexBuffer>,
    shader_programs: EngineData<ShaderProgram>,
}

impl GraphicsManager {
    /// Initialise a new graphics device manager
    pub fn init<T: Platform>(platform: &T) -> GraphicsManager {
        platform.load_gl(gl::load_with);
        
        gl::ClearColor(0.3,0.3,0.3,1.0);
        
        GraphicsManager {
            vertex_buffers: EngineData::new(),
            shader_programs: EngineData::new()
        }
    }

    pub fn shutdown(&mut self) {
        self.vertex_buffers.clear();
        self.shader_programs.clear();
    }

    pub fn clear(&self) {
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }

    pub fn add_vertex_buffer<T>(&mut self, data: ~[f32], stride: u32) -> Option<VertexBufferHandle> {
        VertexBuffer::new(data, stride)
            .and_then(|buffer| {
                Some(self.vertex_buffers.add(buffer))
            })
    }

    pub fn destroy_vertex_buffer(&mut self, handle: Handle) {
        self.vertex_buffers.remove(handle);
    }

    pub fn add_shader_program(&mut self, vert_src: &str, frag_src: &str) -> Option<ShaderProgramHandle> {
        ShaderProgram::new(vert_src, frag_src)
            .and_then(|program| {
                Some(self.shader_programs.add(program))
            })
    }

    pub fn destroy_shader_program(&mut self, handle: Handle) {
        self.shader_programs.remove(handle);
    }

    pub fn cube(width: uint, height: uint, depth: uint) -> Option<VertexBufferHandle> {
        None
    }

    pub fn patch(width: uint, height: uint) -> Option<VertexBufferHandle> {
        None
    }
}

impl Drop for GraphicsManager {
    fn drop(&mut self) {
        // Clean up all the things
        self.shutdown()
    }
}
