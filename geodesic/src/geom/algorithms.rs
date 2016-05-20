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
        let mut new_positions: HashMap<EdgeIndex, PositionIndex> = HashMap::new();
        let mut split_edges: HashMap<EdgeIndex, (EdgeIndex, EdgeIndex)> = HashMap::new();

        let mut mesh = Mesh::empty();
        mesh.positions.extend_from_slice(&self.positions);

        // Create Points for all mid points
        for (index, edge) in self.edges.iter().enumerate() {
            if new_positions.contains_key(&index) {
                continue;
            }

            let mp_index = mesh.add_position(self.edge_midpoint(edge, midpoint_fn));

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
            let p3 = *new_positions.get(&in_e0).unwrap();
            let p4 = *new_positions.get(&in_e1).unwrap();
            let p5 = *new_positions.get(&in_e2).unwrap();

            let _f0 = mesh.add_triangle(p0, p3, p5);
            let _f1 = mesh.add_triangle(p3, p1, p4);
            let _f2 = mesh.add_triangle(p3, p4, p5);
            let _f3 = mesh.add_triangle(p5, p4, p2);

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
            let ref edge = self.edges[*index];
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
