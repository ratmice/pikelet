// Copyright Brendan Zabarauskas 2014

use nalgebra::*;
use noise::source::Source;
use std::rand::Rng;

use World;
use terrain::Terrain;

pub struct Range {
    pub min: f32,
    pub max: f32,
}

impl Range {
    pub fn delta(&self) -> f32 {
        self.max - self.min
    }

    /// Shift a range factor into the range. It is assumed that `factor` is a
    /// number in the range `[0.0, 1.0]`.
    pub fn shift(&self, factor: f32) -> f32 {
        (factor * self.delta()) + self.min
    }
}

/// Construct a model matrix
fn model_mat(scale_x: f32, scale_y: f32, scale_z: f32, position: Pnt3<f32>) -> Mat4<f32> {
    let mut model: Mat4<f32> = zero();
    model.set_col(0, Vec4::x() * scale_x);
    model.set_col(1, Vec4::y() * scale_y);
    model.set_col(2, Vec4::z() * scale_z);
    model.set_col(3, position.to_homogeneous().to_vec());
    model
}

pub struct Scatter {
    pub scale_x: Range,
    pub scale_y: Range,
    pub scale_z: Range,
    pub pos_x:  Range,
    pub pos_y:  Range,
}

impl Scatter {
    pub fn new() -> Scatter {
        Scatter {
            scale_x: Range { min: 0.0, max: 1.0 },
            scale_y: Range { min: 0.0, max: 1.0 },
            scale_z: Range { min: 0.0, max: 1.0 },
            pos_x:  Range { min: 0.0, max: 1.0 },
            pos_y:  Range { min: 0.0, max: 1.0 },
        }
    }

    pub fn scale_x(self, scale_x: Range) -> Scatter {
        Scatter { scale_x: scale_x, ..self }
    }

    pub fn scale_y(self, scale_y: Range) -> Scatter {
        Scatter { scale_y: scale_y, ..self }
    }

    pub fn scale_z(self, scale_z: Range) -> Scatter {
        Scatter { scale_z: scale_z, ..self }
    }

    pub fn pos_x(self, pos_x:  Range) -> Scatter {
        Scatter { pos_x: pos_x, ..self }
    }

    pub fn pos_y(self, pos_y:  Range) -> Scatter {
        Scatter { pos_y: pos_y, ..self }
    }

    pub fn scatter<S: Source, R: Rng>(self, count: uint, terrain: &Terrain<S>, rng: &mut R) -> Objects {
        Objects {
            transforms: {
                range(0, count).map(|_| {
                    let scale_x = self.scale_x.shift(rng.gen());
                    let scale_y = self.scale_y.shift(rng.gen());
                    let scale_z = self.scale_z.shift(rng.gen());
                    let pos_x = self.pos_x.shift(rng.gen());
                    let pos_y = self.pos_y.shift(rng.gen());
                    let pos_z = terrain.get_height_at(pos_x, pos_y);
                    model_mat(scale_x, scale_y, scale_z, Pnt3::new(pos_x, pos_y, pos_z))
                })
                .collect()
            },
        }
    }
}

pub struct Objects {
    pub transforms: Vec<Mat4<f32>>,
}

impl Objects {
    pub fn map_worlds(&self, sun_dir: Vec3<f32>, view_proj: Mat4<f32>, f: |&World|) {
        let mut world = World {
            sun_dir: sun_dir,
            model: one(),
            view_proj: view_proj,
        };

        for model in self.transforms.iter() {
            world.model = *model;
            f(&world)
        }
    }
}
