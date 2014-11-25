// Copyright Brendan Zabarauskas 2014

use nalgebra::*;
use noise::source::Source;
use genmesh::Vertices;
use genmesh::generators::{SharedVertex, SharedVertexIterator};
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

    pub fn shared_pnts<'a, P: SharedVertex<(f32, f32)>>(&'a self, polygon: &'a P) -> TerrainPnts<'a, S, P> {
        TerrainPnts {
            terrain: self,
            vertices: polygon.shared_vertex_iter(),
        }
    }
}

struct TerrainPnts<'a, S: 'a, P: 'a> {
    terrain: &'a Terrain<S>,
    vertices: SharedVertexIterator<'a, P, (f32, f32)>,
}

impl<'a, S: Source, P: SharedVertex<(f32, f32)>> Iterator<Pnt3<f32>> for TerrainPnts<'a, S, P> {
    fn next(&mut self) -> Option<Pnt3<f32>> {
        self.vertices.next().map(|(x, y)| {
            Pnt3::new(x * self.terrain.grid_spacing,
                      y * self.terrain.grid_spacing,
                      self.terrain.source.get(x, y, Float::zero()) * self.terrain.height_factor)
        })
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.vertices.size_hint()
    }
}
