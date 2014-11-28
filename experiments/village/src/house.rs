// Copyright Brendan Zabarauskas 2014

use nalgebra::*;
use noise::source::Source;
use std::rand::Rng;

use shader::Params;
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
    pub positions: Vec<Pnt3<f32>>,
}

impl Village {
    pub fn new<S: Source, R: Rng>(n: uint, scatter_factor: f32, terrain: &Terrain<S>, rng: &mut R) -> Village {
        Village {
            positions: {
                range(0, n).map(|_| {
                    let (x, y) = rng.gen::<(f32, f32)>();
                    let x = (x * scatter_factor) - (scatter_factor / 2.0);
                    let y = (y * scatter_factor) - (scatter_factor / 2.0);
                    let h = terrain.get_height_at(x, y);
                    Pnt3::new(x, y, h)
                })
                .collect()
            },
        }
    }

    pub fn params(&self, sun_dir: [f32, ..3], view_proj: [[f32, ..4], ..4], f: |&Params|) {
        let mut params = Params {
            sun_dir: sun_dir,
            model: *one::<Mat4<_>>().as_array(),
            view_proj: view_proj,
        };

        for pos in self.positions.iter() {
            let mut model = one::<Mat4<f32>>();
            model.set_col(3, pos.to_homogeneous().to_vec());
            params.model = *model.as_array();

            f(&params)
        }
    }
}
