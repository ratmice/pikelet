use cgmath::conv::*;
use geomath::GeoPoint;
use rand::{self, Rng};
use std::sync::mpsc::Sender;

use geom::Mesh;
use geom::primitives;
use geom::algorithms::{Subdivide, Dual};
use math;
use render::{ResourceEvent, Vertex, Indices};

fn generate_planet_mesh(subdivs: usize) -> Mesh {
    primitives::icosahedron(1.0)
        .subdivide(subdivs, &|a, b| math::midpoint_arc(1.0, a, b))
        .generate_dual()
}

fn create_vertices(mesh: &Mesh) -> Vec<Vertex> {
    const VERTICES_PER_FACE: usize = 3;

    let mut vertices = Vec::with_capacity(mesh.faces.len() * VERTICES_PER_FACE);

    for face in &mesh.faces {
        let e0 = face.root;
        let e1 = mesh.edges[e0].next;
        let e2 = mesh.edges[e1].next;

        let p0 = mesh.edges[e0].position;
        let p1 = mesh.edges[e1].position;
        let p2 = mesh.edges[e2].position;

        vertices.push(Vertex { position: mesh.positions[p0].into() });
        vertices.push(Vertex { position: mesh.positions[p1].into() });
        vertices.push(Vertex { position: mesh.positions[p2].into() });
    }

    vertices
}

fn create_star_vertices(count: usize) -> Vec<Vertex> {
    let mut rng = rand::weak_rng();

    (0..count)
        .map(|_| rng.gen::<GeoPoint<f32>>())
        .map(|star| Vertex { position: array3(star.to_point(1.0)) })
        .collect()
}

#[derive(Debug)]
pub enum Job {
    Planet { subdivs: usize },
    Stars { index: usize, count: usize },
}

impl Job {
    pub fn process(self, resource_tx: &Sender<ResourceEvent>) {
        match self {
            Job::Planet { subdivs } => {
                let mesh = generate_planet_mesh(subdivs);
                let vertices = create_vertices(&mesh);

                resource_tx.send(ResourceEvent::UploadBuffer {
                    name: "planet".to_string(),
                    vertices: vertices,
                    indices: Indices::TrianglesList,
                }).unwrap();
            },
            Job::Stars { index, count } => {
                resource_tx.send(ResourceEvent::UploadBuffer {
                    name: format!("stars{}", index),
                    vertices: create_star_vertices(count),
                    indices: Indices::Points,
                }).unwrap();
            },
        }
    }
}

impl PartialEq for Job {
    fn eq(&self, other: &Job) -> bool {
        match (self, other) {
            (&Job::Planet { .. }, &Job::Planet { .. }) => true,
            (&Job::Stars { index: i, .. }, &Job::Stars { index: j, .. }) => i == j,
            (&_, &_) => false,
        }
    }
}
