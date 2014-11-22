// Copyright Brendan Zabarauskas 2014

use nalgebra::*;
use std::mem;

#[shader_param(WorldBatch)]
pub struct Params {
    #[name = "u_Model"]
    pub model: [[f32, ..4], ..4],

    #[name = "u_View"]
    pub view: [[f32, ..4], ..4],

    #[name = "u_Proj"]
    pub proj: [[f32, ..4], ..4],
}

pub struct World {
    pub model: Mat4<f32>,
    pub view:  Mat4<f32>,
    pub proj: PerspMat3<f32>,
}

impl World {
    pub fn as_params(&self) -> &Params {
        unsafe { mem::transmute(self) }
    }
}

