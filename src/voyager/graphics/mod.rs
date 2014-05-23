extern crate gl;
extern crate native;
extern crate cgmath;

use self::gl::types::*;
use std::u16;
use std::vec::Vec;
use std::mem;
use std::ptr;
use std::str;

use cgmath::vector::Vector3;

use platform::Platform;

use super::data::Handle;
use super::data::EngineData;

trait GLObject {
    fn bind();
}

struct VertexBuffer {
    id: GLuint
}

struct ShaderProgram {
    id: GLuint,
    vertex_shader: GLuint,
    fragment_shader: GLuint
}

fn compile_shader(src: &str, shader_type: GLenum) -> Option<GLuint> {
    let id = gl::CreateShader(shader_type);
    unsafe {
        src.with_c_str(|ptr| gl::ShaderSource(id, 1, &ptr, ptr::null()));
        gl::CompileShader(id);

        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut status);

        if status != (gl::TRUE as GLint) {
            None
        } else {
            Some(id)
        }
    }
}

impl ShaderProgram {
    fn new(vert_src: &str, frag_src: &str) -> Option<ShaderProgram> {
        let vs = compile_shader(vert_src, gl::VERTEX_SHADER);
        let fs = compile_shader(frag_src, gl::FRAGMENT_SHADER);

        if vs.is_none() || fs.is_none() {
            None
        } else {
            let id = gl::CreateProgram();
            gl::AttachShader(id, vs.unwrap());
            gl::AttachShader(id, fs.unwrap());
            gl::LinkProgram(id);
            unsafe {
                let mut status = gl::FALSE as GLint;

                if status == (gl::TRUE as GLint) {
                    Some(ShaderProgram{
                        id: id,
                        vertex_shader: vs.unwrap(),
                        fragment_shader: fs.unwrap()
                    })
                } else {
                    let mut len: GLint = 0;
                    gl::GetProgramiv(id, gl::INFO_LOG_LENGTH, &mut len);
                    let mut buf = Vec::from_elem(len as uint - 1, 0u8);
                    gl::GetProgramInfoLog(id, len, ptr::mut_null(), buf.as_mut_ptr() as *mut GLchar);
                    println!("{}", str::from_utf8(buf.as_slice()).expect("ProgramInfoLog invalid."));
                    None
                }
                
            }
        }
    }

    // TODO: uniform access, attributes, parameters, etc etc
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        gl::DeleteProgram(self.id);
        gl::DeleteShader(self.vertex_shader);
        gl::DeleteShader(self.fragment_shader);
    }
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

    pub fn add_vertex_buffer<T>(&mut self, data: ~[T], stride: u32) -> Option<VertexBufferHandle> {
        fail!("Not yet implemented.");
    }

    pub fn destroy_vertex_buffer(&mut self, handle: Handle) {
        fail!("Not yet implemented.");
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
