//! The algorithms module contains traits and implementations of rougly any
//! algorithm that can be understood as a Function `Mesh -> Mesh`.
//!


use std::collections::HashMap;
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
        const RESERVATION_FACTOR: usize = 4;

        let mut new_positions: HashMap<EdgeIndex, PositionIndex> = HashMap::new();
        let mut split_edges: HashMap<EdgeIndex, (EdgeIndex, EdgeIndex)> = HashMap::new();

        let mut positions = self.positions.clone();
        let mut edges = Vec::with_capacity(self.edges.len() * RESERVATION_FACTOR);
        let mut faces = Vec::with_capacity(self.faces.len() * RESERVATION_FACTOR);

        // Create Points for all mid points
        for (index, edge) in self.edges.iter().enumerate() {
            if new_positions.contains_key(&index) {
                continue;
            }

            let mp = self.edge_midpoint(edge, midpoint_fn);
            let mp_index = positions.len();
            positions.push(mp);

            new_positions.insert(index, mp_index);

            if let Some(adjacent_index) = edge.adjacent {
                new_positions.insert(adjacent_index, mp_index);
            }
        }

        // Create our new faces
        for face in self.faces.iter() {
            let in_e0 = face.root;
            let in_e1 = self.edges[in_e0].next;
            let in_e2 = self.edges[in_e1].next;

            debug_assert_eq!(self.edges[in_e2].next, in_e0);

            // New face indices
            let f0 = faces.len();
            let f1 = f0 + 1;
            let f2 = f0 + 2;
            let f3 = f0 + 3;

            // Edge indices: f0     // Edge indices: f1
            let e0 = edges.len();   let e3 = e0 + 3;
            let e1 = e0 + 1;        let e4 = e0 + 4;
            let e2 = e0 + 2;        let e5 = e0 + 5;

            // Edge indices: f2     // Edge indices: f3
            let e6 = e0 + 6;        let  e9 = e0 +  9;
            let e7 = e0 + 7;        let e10 = e0 + 10;
            let e8 = e0 + 8;        let e11 = e0 + 11;

            // Original position indices
            let p0 = self.edges[in_e0].position;
            let p1 = self.edges[in_e1].position;
            let p2 = self.edges[in_e2].position;

            // Midpoint position indices
            let p3 = *new_positions.get(&in_e0).unwrap();
            let p4 = *new_positions.get(&in_e1).unwrap();
            let p5 = *new_positions.get(&in_e2).unwrap();

            faces.push(make_face(f0, e0,  e1,  e2, p0, p3, p5, &mut edges)); // face 0
            faces.push(make_face(f1, e3,  e4,  e5, p3, p1, p4, &mut edges)); // face 1
            faces.push(make_face(f2, e6,  e7,  e8, p3, p4, p5, &mut edges)); // face 2
            faces.push(make_face(f3, e9, e10, e11, p5, p4, p2, &mut edges)); // face 3


            split_edges.insert(in_e0, ( e0,  e3));
            split_edges.insert(in_e1, ( e4, e10));
            split_edges.insert(in_e2, (e11,  e2));


            ///////////////////////////////////////////////////////////////
            // Setup adjacency for internal edges (info we already have)

            // Face 0
            // edges[e0]
            edges[e1].adjacent = Some(e8);
            // edges[e2]

            // Face 1
            // edges[e3]
            // edges[e4]
            edges[e5].adjacent = Some(e6);

            // Face 2
            edges[e6].adjacent = Some(e5);
            edges[e7].adjacent = Some(e9);
            edges[e8].adjacent = Some(e1);

            // Face 3
            edges[e9].adjacent = Some(e7);
            // edges[e10]
            // edges[e11]
        }

        debug_assert_eq!(split_edges.len(), self.edges.len());

        // Update adjacency for remaining edges
        for (index, &(a, b)) in split_edges.iter() {
            let ref edge = self.edges[*index];
            if edge.is_boundary() {
                continue;
            }
            let adjacent_edge = edge.adjacent.unwrap();
            let &(b_adjacent, a_adjacent) = split_edges.get(&adjacent_edge).unwrap();
            edges[a].adjacent = Some(a_adjacent);
            edges[b].adjacent = Some(b_adjacent);
        }

        Mesh {
            positions: positions,
            faces: faces,
            edges: edges,
        }
    }
}

/// This is a bit of a mess, but given three `EdgeIndex`es this will
/// modify the provided `Vec<Edge>` and return a new `Face` from the
/// root edge, effectively "adding a face" but yesh this should be
/// handled differently.
fn make_face(f: FaceIndex, e0: EdgeIndex, e1: EdgeIndex, e2: EdgeIndex,
             p0: PositionIndex, p1: PositionIndex, p2: PositionIndex,
             edges: &mut Vec<Edge>) -> Face
{
    edges.push(Edge::new_boundary(p0, f, e1));
    assert_eq!(edges.len(), e0 + 1);
    edges.push(Edge::new_boundary(p1, f, e2));
    assert_eq!(edges.len(), e1 + 1);
    edges.push(Edge::new_boundary(p2, f, e0));
    assert_eq!(edges.len(), e2 + 1);

    Face::new(e0)
}
