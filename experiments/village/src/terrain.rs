// Copyright Brendan Zabarauskas 2014

use nalgebra::*;
use noise::source::Source;
use genmesh::{Triangle, MapVertex, TriangulateIterator, Triangulate};
use genmesh::generators::Plane;
use std::num::Float;

pub struct Terrain<S> {
    pub height_factor: f32,
    pub grid_spacing: f32,
    pub source: S,
}

impl<S: Source> Terrain<S> {
    pub fn new(height_factor: f32, grid_spacing: f32, source: S) -> Terrain<S> {
        Terrain {
            height_factor:  height_factor,
            grid_spacing:   grid_spacing,
            source:         source,
        }
    }

    pub fn get_height_at(&self, x: f32, y: f32) -> f32 {
        self.source.get(x / self.grid_spacing,
                        y / self.grid_spacing,
                        Float::zero()) * self.height_factor
    }

    pub fn triangulate<'a>(&'a self, polygon: Plane) -> TerrainTriangles<'a, S> {
        TerrainTriangles {
            terrain: self,
            triangles: polygon.triangulate(),
        }
    }
}

struct TerrainTriangles<'a, S: 'a> {
    terrain: &'a Terrain<S>,
    triangles: TriangulateIterator<Plane, (f32, f32)>,
}

impl<'a, S: Source> Iterator<Triangle<(Pnt3<f32>, Vec3<f32>)>> for TerrainTriangles<'a, S> {
    fn next(&mut self) -> Option<Triangle<(Pnt3<f32>, Vec3<f32>)>> {
        fn sub_pnts<T: BaseFloat>(a: &Pnt3<T>, b: &Pnt3<T>) -> Vec3<T> {
            Vec3::new(a.x - b.x, a.y - b.y, a.z - b.z)
        }

        self.triangles.next()
            .map(|tri| tri.map_vertex(|(x, y)| Pnt3 {
                x: x * self.terrain.grid_spacing,
                y: y * self.terrain.grid_spacing,
                z: self.terrain.source.get(x, y, Float::zero()) * self.terrain.height_factor,
            }))
            .map(|Triangle { x, y, z }| {
                let v = sub_pnts(&y, &x);          // first side of the triangle
                let w = sub_pnts(&z, &x);          // second side of the triangle
                let n = normalize(&cross(&v, &w)); // the normal of the triangle
                Triangle { x: (x, n), y: (y, n), z: (z, n) }
            })
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.triangles.size_hint()
    }
}
