// Copyright Brendan Zabarauskas 2014

use nalgebra::*;
use noise::source::Source;
use std::rand::Rng;

use World;
use shader::flat::Vertex;
use terrain::Terrain;

pub const VERTEX_DATA: &'static [Vertex] = &[
    // top (0, 0, 1)
    Vertex { pos: [-1.0, -1.0,  1.0], norm: [ 0.0,  0.0,  1.0], color: [1.0, 1.0, 1.0] },
    Vertex { pos: [ 1.0, -1.0,  1.0], norm: [ 0.0,  0.0,  1.0], color: [1.0, 1.0, 1.0] },
    Vertex { pos: [ 1.0,  1.0,  1.0], norm: [ 0.0,  0.0,  1.0], color: [1.0, 1.0, 1.0] },
    Vertex { pos: [-1.0,  1.0,  1.0], norm: [ 0.0,  0.0,  1.0], color: [1.0, 1.0, 1.0] },
    // bottom (0, 0, -1)
    Vertex { pos: [-1.0,  1.0, -1.0], norm: [ 0.0,  0.0, -1.0], color: [1.0, 1.0, 1.0] },
    Vertex { pos: [ 1.0,  1.0, -1.0], norm: [ 0.0,  0.0, -1.0], color: [1.0, 1.0, 1.0] },
    Vertex { pos: [ 1.0, -1.0, -1.0], norm: [ 0.0,  0.0, -1.0], color: [1.0, 1.0, 1.0] },
    Vertex { pos: [-1.0, -1.0, -1.0], norm: [ 0.0,  0.0, -1.0], color: [1.0, 1.0, 1.0] },
    // right (1, 0, 0)
    Vertex { pos: [ 1.0, -1.0, -1.0], norm: [ 1.0,  0.0,  0.0], color: [1.0, 1.0, 1.0] },
    Vertex { pos: [ 1.0,  1.0, -1.0], norm: [ 1.0,  0.0,  0.0], color: [1.0, 1.0, 1.0] },
    Vertex { pos: [ 1.0,  1.0,  1.0], norm: [ 1.0,  0.0,  0.0], color: [1.0, 1.0, 1.0] },
    Vertex { pos: [ 1.0, -1.0,  1.0], norm: [ 1.0,  0.0,  0.0], color: [1.0, 1.0, 1.0] },
    // left (-1, 0, 0)
    Vertex { pos: [-1.0, -1.0,  1.0], norm: [-1.0,  0.0,  0.0], color: [1.0, 1.0, 1.0] },
    Vertex { pos: [-1.0,  1.0,  1.0], norm: [-1.0,  0.0,  0.0], color: [1.0, 1.0, 1.0] },
    Vertex { pos: [-1.0,  1.0, -1.0], norm: [-1.0,  0.0,  0.0], color: [1.0, 1.0, 1.0] },
    Vertex { pos: [-1.0, -1.0, -1.0], norm: [-1.0,  0.0,  0.0], color: [1.0, 1.0, 1.0] },
    // front (0, 1, 0)
    Vertex { pos: [ 1.0,  1.0, -1.0], norm: [ 0.0,  1.0,  0.0], color: [1.0, 1.0, 1.0] },
    Vertex { pos: [-1.0,  1.0, -1.0], norm: [ 0.0,  1.0,  0.0], color: [1.0, 1.0, 1.0] },
    Vertex { pos: [-1.0,  1.0,  1.0], norm: [ 0.0,  1.0,  0.0], color: [1.0, 1.0, 1.0] },
    Vertex { pos: [ 1.0,  1.0,  1.0], norm: [ 0.0,  1.0,  0.0], color: [1.0, 1.0, 1.0] },
    // back (0, -1, 0)
    Vertex { pos: [ 1.0, -1.0,  1.0], norm: [ 0.0, -1.0,  0.0], color: [1.0, 1.0, 1.0] },
    Vertex { pos: [-1.0, -1.0,  1.0], norm: [ 0.0, -1.0,  0.0], color: [1.0, 1.0, 1.0] },
    Vertex { pos: [-1.0, -1.0, -1.0], norm: [ 0.0, -1.0,  0.0], color: [1.0, 1.0, 1.0] },
    Vertex { pos: [ 1.0, -1.0, -1.0], norm: [ 0.0, -1.0,  0.0], color: [1.0, 1.0, 1.0] },
];

pub const INDEX_DATA: &'static [u8] = &[
     0,  1,  2,  2,  3,  0, // top
     4,  5,  6,  6,  7,  4, // bottom
     8,  9, 10, 10, 11,  8, // right
    12, 13, 14, 14, 15, 12, // left
    16, 17, 18, 18, 19, 16, // front
    20, 21, 22, 22, 23, 20, // back
];

pub struct Village {
    pub transforms: Vec<Mat4<f32>>,
}

fn model_mat(scale_x: f32, scale_y: f32, scale_z: f32, position: Pnt3<f32>) -> Mat4<f32> {
    let mut model: Mat4<f32> = zero();
    model.set_col(0, Vec4::x() * scale_x);
    model.set_col(1, Vec4::y() * scale_y);
    model.set_col(2, Vec4::z() * scale_z);
    model.set_col(3, position.to_homogeneous().to_vec());
    model
}

impl Village {
    pub fn new<S: Source, R: Rng>(n: uint, scatter_factor: f32, terrain: &Terrain<S>, rng: &mut R) -> Village {
        Village {
            transforms: {
                range(0, n).map(|_| {
                    let (x, y) = rng.gen::<(f32, f32)>();
                    let x = (x * scatter_factor) - (scatter_factor / 2.0);
                    let y = (y * scatter_factor) - (scatter_factor / 2.0);
                    let z = terrain.get_height_at(x, y);
                    let scale_x = (rng.gen::<f32>() * 10.0) + 1.0;
                    let scale_y = (rng.gen::<f32>() * 10.0) + 1.0;
                    let scale_z = (rng.gen::<f32>() * 10.0) + 1.0;
                    model_mat(scale_x, scale_y, scale_z, Pnt3::new(x, y, z))
                })
                .collect()
            },
        }
    }

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
