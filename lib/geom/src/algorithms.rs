//! The algorithms module contains traits and implementations of rougly any
//! algorithm that can be understood as a Function `Mesh -> Mesh`.

use cgmath::prelude::*;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use fnv::FnvHasher;

use {EdgeIndex, FaceIndex, Mesh, Position, PositionIndex};

type MidpointCache = HashMap<EdgeIndex, PositionIndex, BuildHasherDefault<FnvHasher>>;
type SplitEdgeCache = HashMap<EdgeIndex, (EdgeIndex, EdgeIndex), BuildHasherDefault<FnvHasher>>;

/// Trait for types that support being subdivided.
pub trait Subdivide {
    /// Applies `subdivide_once` the specified number of times.
    fn subdivide<F>(&self, count: usize, midpoint_fn: &F) -> Mesh
        where F: Fn(Position, Position) -> Position;
    /// The actual subdivision implementation.
    fn subdivide_once<F>(&self, midpoint_fn: &F) -> Mesh where F: Fn(Position, Position) -> Position;
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
        (0..count).fold(self.clone(), |acc, _| acc.subdivide_once(&midpoint_fn))
    }

    fn subdivide_once<F>(&self, midpoint_fn: &F) -> Mesh
        where F: Fn(Position, Position) -> Position
    {
        fn calc_and_cache_midpoint<F>(index: EdgeIndex,
                                      in_mesh: &Mesh,
                                      out_mesh: &mut Mesh,
                                      cache: &mut MidpointCache,
                                      midpoint_fn: &F)
                                      -> PositionIndex
            where F: Fn(Position, Position) -> Position
        {
            let edge = in_mesh.edge(index).unwrap();
            let mp_index = out_mesh.add_position(in_mesh.edge_midpoint(edge, midpoint_fn));
            if let Some(adjacent_index) = edge.adjacent {
                cache.insert(adjacent_index, mp_index);
            }
            mp_index
        }

        let mut midpoint_cache = {
            let fnv = BuildHasherDefault::<FnvHasher>::default();
            MidpointCache::with_hasher(fnv)
        };
        let mut split_edges = {
            let fnv = BuildHasherDefault::<FnvHasher>::default();
            SplitEdgeCache::with_hasher(fnv)
        };

        let mut mesh = Mesh::empty();
        mesh.positions.extend_from_slice(&self.positions);

        // Create our new faces
        for face in &self.faces {
            let in_e0 = face.root;
            let in_e1 = self.edge(in_e0).unwrap().next;
            let in_e2 = self.edge(in_e1).unwrap().next;

            debug_assert_eq!(self.edge(in_e2).unwrap().next, in_e0);

            // Original position indices
            let p0 = self.edge(in_e0).unwrap().position;
            let p1 = self.edge(in_e1).unwrap().position;
            let p2 = self.edge(in_e2).unwrap().position;

            // Midpoint position indices

            let p3 = match midpoint_cache.remove(&in_e0) {
                Some(point) => point,
                None => {
                    calc_and_cache_midpoint(in_e0,
                                            self,
                                            &mut mesh,
                                            &mut midpoint_cache,
                                            &midpoint_fn)
                },
            };

            let p4 = match midpoint_cache.remove(&in_e1) {
                Some(point) => point,
                None => {
                    calc_and_cache_midpoint(in_e1,
                                            self,
                                            &mut mesh,
                                            &mut midpoint_cache,
                                            &midpoint_fn)
                },
            };

            let p5 = match midpoint_cache.remove(&in_e2) {
                Some(point) => point,
                None => {
                    calc_and_cache_midpoint(in_e2,
                                            self,
                                            &mut mesh,
                                            &mut midpoint_cache,
                                            &midpoint_fn)
                },
            };

            let (_, (e0, e1, e2)) = mesh.add_triangle(p0, p3, p5);
            let (_, (e3, e4, e5)) = mesh.add_triangle(p3, p1, p4);
            let (_, (e6, e7, e8)) = mesh.add_triangle(p3, p4, p5);
            let (_, (e9, e10, e11)) = mesh.add_triangle(p5, p4, p2);

            split_edges.insert(in_e0, (e0, e3));
            split_edges.insert(in_e1, (e4, e10));
            split_edges.insert(in_e2, (e11, e2));

            mesh.make_adjacent(e1, e8);
            mesh.make_adjacent(e5, e6);
            mesh.make_adjacent(e7, e9);
        }

        debug_assert_eq!(split_edges.len(), self.edges.len());

        // Update adjacency for remaining edges
        for (index, &(a, b)) in &split_edges {
            let edge = self.edge(*index).unwrap();
            if edge.is_boundary() {
                continue;
            }
            let adjacent_edge = edge.adjacent.unwrap();
            let (b_adjacent, a_adjacent) = split_edges[&adjacent_edge];
            mesh.edge_mut(a).unwrap().adjacent = Some(a_adjacent);
            mesh.edge_mut(b).unwrap().adjacent = Some(b_adjacent);
        }

        mesh
    }
}

type CentroidCache = HashMap<FaceIndex, PositionIndex, BuildHasherDefault<FnvHasher>>;
type FaceMembershipCache = HashMap<PositionIndex, FaceIndex, BuildHasherDefault<FnvHasher>>;

pub trait Dual {
    fn generate_dual(&self) -> Mesh;
}

impl Dual for Mesh {
    #[cfg_attr(feature = "cargo-clippy", allow(should_assert_eq))]
    fn generate_dual(&self) -> Mesh {
        fn next_face_around_position(mesh: &Mesh, pi: PositionIndex, ei0: EdgeIndex) -> FaceIndex {
            let e0 = mesh.edge(ei0).unwrap();
            let e1 = mesh.edge(e0.next).unwrap();
            let e2 = mesh.edge(e1.next).unwrap();

            debug_assert_eq!(e0.position, pi);
            debug_assert_eq!(e2.next, ei0);

            debug_assert!(!e2.is_boundary());

            let adjacent_edge = mesh.edge(e2.adjacent.unwrap()).unwrap();
            adjacent_edge.face
        }

        let mut centroid_cache = {
            let fnv = BuildHasherDefault::<FnvHasher>::default();
            CentroidCache::with_hasher(fnv)
        };
        let mut face_cache = {
            let fnv = BuildHasherDefault::<FnvHasher>::default();
            FaceMembershipCache::with_hasher(fnv)
        };

        let mut mesh = Mesh::empty();

        for (face_index, point_indices) in self.triangles().enumerate() {
            let face_index = FaceIndex(face_index);

            face_cache.entry(point_indices[0]).or_insert(face_index);
            face_cache.entry(point_indices[1]).or_insert(face_index);
            face_cache.entry(point_indices[2]).or_insert(face_index);

            let face_positions = [*self.position(point_indices[0]).unwrap(),
                                  *self.position(point_indices[1]).unwrap(),
                                  *self.position(point_indices[2]).unwrap()];

            let centroid_index = mesh.add_position(Position::centroid(&face_positions));
            centroid_cache.insert(face_index, centroid_index);
        }

        for pi in (0..self.positions.len()).map(PositionIndex) {
            let fi0 = *face_cache
                           .get(&pi)
                           .expect("Position in Mesh without any connected faces!?");
            let mut current_face_index = fi0;

            let mut centroids = Vec::with_capacity(6);
            let mut centroid_indices = Vec::with_capacity(6);

            loop {
                let current_face = self.face(current_face_index).unwrap();
                let centroid_index = centroid_cache[&current_face_index];

                centroid_indices.push(centroid_index);
                centroids.push(*mesh.position(centroid_index).unwrap());

                let ei = {
                    let e0 = self.edge(current_face.root).unwrap();
                    if e0.position == pi {
                        current_face.root
                    } else {
                        let e1 = self.edge(e0.next).unwrap();
                        if e1.position == pi {
                            e0.next
                        } else {
                            assert!(pi == self.edge(e1.next).unwrap().position,
                                    "Unable to find outgoing edge for {:?}",
                                    pi);
                            e1.next
                        }
                    }
                };
                current_face_index = next_face_around_position(self, pi, ei);
                if current_face_index == fi0 {
                    break;
                }

                // Make sure we don't spin out of control
                assert!(centroids.len() <= 6, "Infinite loop detected!");
            }

            let centroid = Position::centroid(&centroids);
            let centroid_index = mesh.add_position(centroid);

            match centroids.len() {
                6 => {
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
