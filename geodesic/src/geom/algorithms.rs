//! The algorithms module contains traits and implementations of rougly any
//! algorithm that can be understood as a Function `Mesh -> Mesh`.
//!


use std::collections::HashMap;
use ::math;
use super::*;

/// Trait for types that support being subdivided.
pub trait Subdivide {
    /// Applies `subdivide_once` the specified number of times.
    fn subdivide<F>(&self, count: usize, midpoint_fn: &F) -> Mesh
        where F: Fn(Position, Position) -> Position;
    /// The actual subdivision implementation.
    fn subdivide_once<F>(&self, midpoint_fn: &F) -> Mesh
        where F: Fn(Position, Position) -> Position;
}


/// Implements Class I subdivision on `geom::Mesh` objects:
///
/// ```text
///           v0         |  v0 ____v5____ v2
///           /\         |    \    /\    /
///          /  \        |     \  /  \  /
///     v3  /____\  v5   |   v3 \/____\/ v4
///        /\    /\      |       \    /
///       /  \  /  \     |        \  /
///      /____\/____\    |         \/
///    v1     v4     v2  |         v1
/// ```
///
/// # Note
///
/// This method will panic if the Mesh has non-triangle faces.
///
impl Subdivide for Mesh {
    fn subdivide<F>(&self, count: usize, midpoint_fn: &F) -> Mesh
        where F: Fn(Position, Position) -> Position
    {
        (0..count).fold(self.clone(), |acc, _| {
            acc.subdivide_once(&midpoint_fn)
        })
    }

    fn subdivide_once<F>(&self, midpoint_fn: &F) -> Mesh
        where F: Fn(Position, Position) -> Position
    {
        fn calc_and_cache_midpoint<F>(index: EdgeIndex, in_mesh: &Mesh, out_mesh: &mut Mesh,
                                      cache: &mut HashMap<EdgeIndex, PositionIndex>,
                                      midpoint_fn: &F) -> PositionIndex
            where F: Fn(Position, Position) -> Position
        {
            let edge = &in_mesh.edges[index];
            let mp_index = out_mesh.add_position(
                in_mesh.edge_midpoint(edge, midpoint_fn)
            );
            if let Some(adjacent_index) = edge.adjacent {
                cache.insert(adjacent_index, mp_index);
            }
            mp_index
        }

        let mut midpoint_cache: HashMap<EdgeIndex, PositionIndex> = HashMap::new();
        let mut split_edges: HashMap<EdgeIndex, (EdgeIndex, EdgeIndex)> = HashMap::new();

        let mut mesh = Mesh::empty();
        mesh.positions.extend_from_slice(&self.positions);

        // Create our new faces
        for face in self.faces.iter() {
            let in_e0 = face.root;
            let in_e1 = self.edges[in_e0].next;
            let in_e2 = self.edges[in_e1].next;

            debug_assert_eq!(self.edges[in_e2].next, in_e0);

            // Edge indices: f0             // Edge indices: f1
            let e0 = mesh.next_edge_id();   let e3 = e0 + 3;
            let e1 = e0 + 1;                let e4 = e0 + 4;
            let e2 = e0 + 2;                let e5 = e0 + 5;

            // Edge indices: f2     // Edge indices: f3
            let e6 = e0 + 6;        let  e9 = e0 +  9;
            let e7 = e0 + 7;        let e10 = e0 + 10;
            let e8 = e0 + 8;        let e11 = e0 + 11;

            // Original position indices
            let p0 = self.edges[in_e0].position;
            let p1 = self.edges[in_e1].position;
            let p2 = self.edges[in_e2].position;

            // Midpoint position indices

            let p3 = midpoint_cache
                .remove(&in_e0)
                .unwrap_or_else(|| {
                    calc_and_cache_midpoint(
                        in_e0, &self, &mut mesh, &mut midpoint_cache, &midpoint_fn
                    )
                });

            let p4 = midpoint_cache
                .remove(&in_e1)
                .unwrap_or_else(|| {
                    calc_and_cache_midpoint(
                        in_e1, &self, &mut mesh, &mut midpoint_cache, &midpoint_fn
                    )
                });

            let p5 = midpoint_cache
                .remove(&in_e2)
                .unwrap_or_else(|| {
                    calc_and_cache_midpoint(
                        in_e2, &self, &mut mesh, &mut midpoint_cache, &midpoint_fn
                    )
                });

            mesh.add_triangle(p0, p3, p5);
            mesh.add_triangle(p3, p1, p4);
            mesh.add_triangle(p3, p4, p5);
            mesh.add_triangle(p5, p4, p2);

            split_edges.insert(in_e0, ( e0,  e3));
            split_edges.insert(in_e1, ( e4, e10));
            split_edges.insert(in_e2, (e11,  e2));

            mesh.make_adjacent(e1, e8);
            mesh.make_adjacent(e5, e6);
            mesh.make_adjacent(e7, e9);
        }

        debug_assert_eq!(split_edges.len(), self.edges.len());

        // Update adjacency for remaining edges
        for (index, &(a, b)) in split_edges.iter() {
            let edge = &self.edges[*index];
            if edge.is_boundary() {
                continue;
            }
            let adjacent_edge = edge.adjacent.unwrap();
            let &(b_adjacent, a_adjacent) = split_edges.get(&adjacent_edge).unwrap();
            mesh.edges[a].adjacent = Some(a_adjacent);
            mesh.edges[b].adjacent = Some(b_adjacent);
        }

        mesh
    }
}


pub trait Dual {
    fn generate_dual(&self) -> Mesh;
}

impl Dual for Mesh {
    fn generate_dual(&self) -> Mesh {
        fn next_face_around_position(mesh: &Mesh, pi: PositionIndex, ei0: EdgeIndex) -> FaceIndex {
            let e0 = &mesh.edges[ei0];
            let e1 = &mesh.edges[e0.next];
            let e2 = &mesh.edges[e1.next];

            debug_assert_eq!(e0.position, pi);
            debug_assert_eq!(e2.next, ei0);

            debug_assert!(!e2.is_boundary());

            let adjacent_edge = &mesh.edges[e2.adjacent.unwrap()];
            adjacent_edge.face
        }

        let mut centroid_cache: HashMap<FaceIndex, PositionIndex> = HashMap::new();
        let mut face_cache: HashMap<PositionIndex, FaceIndex> = HashMap::new();

        let mut mesh = Mesh::empty();

        // TODO: make some iterators!
        for (fi, face) in self.faces.iter().enumerate() {
            let points = {
                let e0 = &self.edges[face.root];
                let e1 = &self.edges[e0.next];
                let e2 = &self.edges[e1.next];

                let p0 = e0.position;
                face_cache.entry(p0).or_insert(fi);
                let p1 = e1.position;
                face_cache.entry(p1).or_insert(fi);
                let p2 = e2.position;
                face_cache.entry(p2).or_insert(fi);

                vec![
                    self.positions[p0],
                    self.positions[p1],
                    self.positions[p2],
                ]
            };

            let cp = mesh.add_position(math::centroid(&points));
            centroid_cache.insert(fi, cp);
        }

        for pi in 0..self.positions.len() {
            let fi0 = *face_cache.get(&pi).expect("Position in Mesh without any connected faces!?");
            let mut current_face_index = fi0;

            let mut centroids = Vec::with_capacity(6);
            let mut centroid_indices = Vec::with_capacity(6);
            let mut cycle_check = 0;

            loop {
                let current_face = &self.faces[current_face_index];
                let centroid_index = centroid_cache[&current_face_index];

                centroid_indices.push(centroid_index);
                centroids.push(mesh.positions[centroid_index]);

                let ei = {
                    let e0 = &self.edges[current_face.root];
                    if e0.position == pi {
                        current_face.root
                    } else {
                        let e1 = &self.edges[e0.next];
                        if e1.position == pi {
                            e0.next
                        } else {
                            let e2 = &self.edges[e1.next];
                            if e2.position == pi {
                                e1.next
                            } else {
                                panic!("Unable to find outgoing edge for position {}", pi);
                            }
                        }
                    }
                };
                current_face_index = next_face_around_position(&self, pi, ei);
                if current_face_index == fi0 {
                    break;
                }

                // Make sure we don't spin out of control
                cycle_check += 1;
                if cycle_check > 6 {
                    panic!("Infinite loop detected!");
                }
            }

            let centroid = math::centroid(&centroids);
            let centroid_index = mesh.add_position(centroid);

            match centroids.len() {
                6 =>  {
                    mesh.add_triangle(centroid_index, centroid_indices[0], centroid_indices[1]);
                    mesh.add_triangle(centroid_index, centroid_indices[1], centroid_indices[2]);
                    mesh.add_triangle(centroid_index, centroid_indices[2], centroid_indices[3]);
                    mesh.add_triangle(centroid_index, centroid_indices[3], centroid_indices[4]);
                    mesh.add_triangle(centroid_index, centroid_indices[4], centroid_indices[5]);
                    mesh.add_triangle(centroid_index, centroid_indices[5], centroid_indices[0]);
                },
                5 => {
                    mesh.add_triangle(centroid_index, centroid_indices[0], centroid_indices[1]);
                    mesh.add_triangle(centroid_index, centroid_indices[1], centroid_indices[2]);
                    mesh.add_triangle(centroid_index, centroid_indices[2], centroid_indices[3]);
                    mesh.add_triangle(centroid_index, centroid_indices[3], centroid_indices[4]);
                    mesh.add_triangle(centroid_index, centroid_indices[4], centroid_indices[0]);
                },
                n => panic!("Incorrect number of centroids: {:?}!", n),
            }
        }

        mesh
    }
}
