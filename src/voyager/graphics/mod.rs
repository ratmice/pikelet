extern crate gl;
extern crate native;

use self::gl::types::*;
use std::u16;
use std::vec::Vec;
use std::mem;
use std::ptr;
use std::str;

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

struct ShaderProgram {
    _id: GLuint,
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
            let program_id = gl::CreateProgram();
            gl::AttachShader(program_id, vs.unwrap());
            gl::AttachShader(program_id, fs.unwrap());
            gl::LinkProgram(program_id);
            unsafe {
                let mut status = gl::FALSE as GLint;

                if status == (gl::TRUE as GLint) {
                    Some(ShaderProgram{
                        _id: program_id,
                        vertex_shader: vs.unwrap(),
                        fragment_shader: fs.unwrap()
                    })
                } else {
                    let mut len: GLint = 0;
                    gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
                    let mut buf = Vec::from_elem(len as uint - 1, 0u8);
                    gl::GetProgramInfoLog(program_id, len, ptr::mut_null(), buf.as_mut_ptr() as *mut GLchar);
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
        gl::DeleteProgram(self._id);
        gl::DeleteShader(self.vertex_shader);
        gl::DeleteShader(self.fragment_shader);
    }
}

#[deriving(Clone, Eq)]
pub struct Handle {
    ref_index: u16,
    generation: u16
}

/// A graphics device manager
pub struct GraphicsManager {
    current_generation: uint,
    vertex_buffers: Vec<VertexBuffer>,
    shader_programs: Vec<ShaderProgram>,
}

impl GraphicsManager {
    /// Initialise a new graphics device manager
    pub fn init<T: Platform>(platform: &T) -> GraphicsManager {
        platform.load_gl(gl::load_with);
        
        gl::ClearColor(0.3,0.3,0.3,1.0);
        
        GraphicsManager {
            current_generation: 0,
            vertex_buffers: Vec::with_capacity(u16::MAX as uint),
            shader_programs: Vec::with_capacity(u16::MAX as uint),
        }
    }

    pub fn shutdown(&mut self) {
        self.vertex_buffers.clear();
        self.shader_programs.clear();
    }

    pub fn clear(&self) {
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }

    pub fn add_vertex_buffer<T>(&mut self, data: ~[T], stride: u32) -> Option<Handle> {
        fail!("Not yet implemented.");
    }

    pub fn destroy_vertex_buffer(&mut self, handle: Handle) {
        fail!("Not yet implemented.");
    }

    pub fn add_shader_program(&mut self, vert_src: &str, frag_src: &str) -> Option<Handle> {
        ShaderProgram::new(vert_src, frag_src)
            .and_then(|program| {
                self.shader_programs.push(program);
                Some(Handle {
                    ref_index: (self.shader_programs.len() - 1) as u16,
                    generation: self.current_generation as u16 // I haven't worked out the details of this yet
                })
            })
    }

    pub fn destroy_shader_program(&mut self, handle: Handle) {
        let next_gen = self.shader_programs.swap_remove(handle.ref_index as uint)
            .and_then(|program| {
                Some(self.current_generation + 1)
            }).unwrap_or(self.current_generation);
        self.current_generation = next_gen;
    }
}

impl Drop for GraphicsManager {
    fn drop(&mut self) {
        // Clean up all the things
        self.shutdown()
    }
}
