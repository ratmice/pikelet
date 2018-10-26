// Copyright The Voyager Developers 2014

use nalgebra::*;
use noise::{GenFn2, Seed};
use genmesh::{Triangle, MapVertex, TriangulateIterator, Triangulate};
use genmesh::generators::Plane;
use std::num::Float;

pub struct Terrain<F: GenFn2<f32>> {
    pub seed: Seed,
    pub function: F,
    pub height_factor: f32,
    pub grid_spacing: f32,
}

impl<F: GenFn2<f32>> Terrain<F> {
    pub fn new(seed: Seed, function: F, height_factor: f32, grid_spacing: f32) -> Terrain<F> {
        Terrain {
            seed:           seed,
            function:       function,
            height_factor:  height_factor,
            grid_spacing:   grid_spacing,
        }
    }

    pub fn get_height_at(&self, x: f32, y: f32) -> f32 {
        (self.function)(&self.seed, &[x / self.grid_spacing, y / self.grid_spacing]) * self.height_factor
    }

    pub fn triangulate<'a>(&'a self, polygon: Plane) -> TerrainTriangles<'a, F> {
        TerrainTriangles {
            terrain: self,
            triangles: polygon.triangulate(),
        }
    }
}

struct TerrainTriangles<'a, F: 'a> {
    terrain: &'a Terrain<F>,
    triangles: TriangulateIterator<Plane, (f32, f32)>,
}

impl<'a, F: GenFn2<f32>> Iterator for TerrainTriangles<'a, F> {
    type Item = Triangle<(Pnt3<f32>, Vec3<f32>)>;

    fn next(&mut self) -> Option<Triangle<(Pnt3<f32>, Vec3<f32>)>> {
        self.triangles.next()
            .map(|tri| tri.map_vertex(|(x, y)| Pnt3 {
                x: x * self.terrain.grid_spacing,
                y: y * self.terrain.grid_spacing,
                z: self.terrain.get_height_at(x, y),
            }))
            .map(|Triangle { x, y, z }| {
                let v = y - x;          // first side of the triangle
                let w = z - x;          // second side of the triangle
                let n = cross(&v, &w);  // the normal of the triangle
                Triangle { x: (x, n), y: (y, n), z: (z, n) }
            })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.triangles.size_hint()
    }
}
