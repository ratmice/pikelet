extern crate gl;
extern crate native;
extern crate cgmath;

use self::gl::types::*;
use std::vec::Vec;
use std::ptr;
use std::str;

use graphics::GLObject;


pub struct ShaderProgram {
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
    pub fn new(vert_src: &str, frag_src: &str) -> Option<ShaderProgram> {
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
                gl::GetProgramiv(id, gl::LINK_STATUS, &mut status);

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

impl GLObject for ShaderProgram {
    fn bind() {}
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        gl::DeleteProgram(self.id);
        gl::DeleteShader(self.vertex_shader);
        gl::DeleteShader(self.fragment_shader);
    }
}
