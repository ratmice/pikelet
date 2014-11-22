// Copyright Brendan Zabarauskas 2014

use nalgebra::*;

pub struct Camera {
    pub from: Vec3<f32>,
    pub target: Vec3<f32>,
    pub project: Persp3<f32>,
}

impl Camera {
    pub fn to_mat4(&self) -> Mat4<f32> {
        unimplemented!()
    }
}
