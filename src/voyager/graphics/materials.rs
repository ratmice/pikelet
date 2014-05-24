extern crate gl;
extern crate cgmath;

use self::gl::types::*;
use cgmath::vector::Vector4;

use graphics::{Bind, ShaderProgramHandle};


pub struct Material {
    program: ShaderProgramHandle,
    diffuse: Vector4<f32>,
    ambient: Vector4<f32>
}

impl Material {
    fn new() {
    }
}

impl Bind for Material {
    fn bind() {}
}
