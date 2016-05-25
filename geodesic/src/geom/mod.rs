//! Module defining principle structs for working with mesh data using the
//! "half edge" data structure.
//! Some liberties have been taken in attempt to adapt this structure to
//! our needs.

use cgmath::Point3;

pub use self::mesh::Mesh;

pub mod mesh;
pub mod algorithms;
pub mod primitives;
pub mod star_field;

pub type EdgeIndex = usize;
pub type PositionIndex = usize;
pub type FaceIndex = usize;
pub type VertexIndex = usize;
pub type Position = Point3<f32>;


///  Face
///
///  TODO: Is the face really so sparse?
///        Probably not! Because there is a bunch of attributes, seeds, values,
///        parameters, references to things, and so on, and so on; that could
///        be associated and organized with a single Face. So let's assume
///        that connectivity aside, we'll be stuffing stuff into the Face struct
///        eventually.
///
#[derive(Clone, Debug)]
pub struct Face {
    /// The index of the first edge to define this face.
    pub root: EdgeIndex,
}

impl Face {
    /// Contructs a new `Face` give a root `EdgeIndex`
    pub fn new(root: EdgeIndex) -> Face {
        Face { root: root }
    }
}


/// Used to build an index of Point/Edge/Face relationships
#[derive(Clone, Debug)]
pub struct Vertex {
    pub edges: Vec<EdgeIndex>,
}


/// Our primary entity for navigating the topology of a Mesh
///
/// Vertices and edges are essentially the same in this data structure
/// So I've deviated a bit from the vernacular and "collapsed" the
/// Vertex and Edge structures into a single struct.
/// It may be that we bring back the Vertex and use that as an index
/// to every conneted edge though.
///
#[derive(Clone, Debug)]
pub struct Edge {
    /// Attribute index for this vertex.
    pub position: PositionIndex,

    /// The face that this edge is associated with.
    pub face: FaceIndex,

    /// The index of the next edge/vert around the face.
    pub next: EdgeIndex,

    /// Oppositely oriented adjacent Edge.
    /// If this is None then we have a boundary edge.
    pub adjacent: Option<EdgeIndex>,
}

impl Edge {
    /// Constructs a new `Edge` including a reference to an adjacent `Edge`.
    pub fn new(point: PositionIndex, face: FaceIndex, next: EdgeIndex, adjacent: EdgeIndex) -> Edge {
        Edge {
            position: point,
            face: face,
            next: next,
            adjacent: Some(adjacent),
        }
    }

    /// Constructs a new `Edge` which has no adjacent edge.
    pub fn new_boundary(point: PositionIndex, face: FaceIndex, next: EdgeIndex) -> Edge {
        Edge {
            position: point,
            face: face,
            next: next,
            adjacent: None,
        }
    }

    /// Simplify check of potential adjacency.
    pub fn is_boundary(&self) -> bool {
        self.adjacent.is_none()
    }
}


////////////////////////////////////////////////////////////////////////////////
// Tests
////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use ::math;
    use super::{Mesh, Edge, EdgeIndex};
    use super::primitives;
    use super::algorithms::*;

    fn assert_congruent_adjacenct_positions(e0: &Edge, e1: &Edge, mesh: &Mesh) {
        let e0p0 = e0.position.clone();
        let e0p1 = mesh.edges[e0.next].position.clone();

        let e1p0 = e1.position.clone();
        let e1p1 = mesh.edges[e1.next].position.clone();

        assert_eq!(e0p0, e1p1);
        assert_eq!(e0p1, e1p0);
    }

    fn assert_congruent_adjacency(index: &EdgeIndex, edge: &Edge, mesh: &Mesh) {
        let adjacent_index = edge.adjacent.unwrap().clone();
        let ref adjacent_edge = mesh.edges[adjacent_index];
        assert!(adjacent_edge.adjacent.is_some());

        let expected_index = adjacent_edge.adjacent.unwrap().clone();
        assert_eq!(*index, expected_index);

        assert_congruent_adjacenct_positions(&edge, &adjacent_edge, &mesh);
    }

    // used to test meshes that should have no boundary edges
    fn assert_congruent_nonboundary_mesh(mesh: &Mesh) {
        for (index, edge) in mesh.edges.iter().enumerate() {
            assert!(edge.adjacent.is_some());
            assert_congruent_adjacency(&index, &edge, &mesh);
        }
    }

    // used to test meshes which are allowed to have boundary edges
    fn assert_congruent_mesh(mesh: &Mesh) {
        for (index, edge) in mesh.edges.iter().enumerate() {
            if edge.adjacent.is_none() {
                continue
            }
            assert_congruent_adjacency(&index, &edge, &mesh);
        }
    }

    fn assert_face_associations(mesh: &Mesh) {
        let mut cycle_check = 0;
        for (fi, face) in mesh.faces.iter().enumerate() {
            let ei0 = face.root;
            let mut eiN = ei0;
            loop {
                let ref edge = mesh.edges[eiN];
                assert_eq!(edge.face, fi);

                eiN = edge.next;
                if eiN == ei0 {
                    break;
                }
                cycle_check += 1;
                if cycle_check > mesh.edges.len() {
                    panic!("Edges around face do not terminate!");
                }
            }
        }
    }

    #[test]
    fn icosahedron() {
        let planet_radius: f32 = 1.0;
        let icosahedron = primitives::icosahedron(planet_radius);
        assert_congruent_nonboundary_mesh(&icosahedron);
        assert_face_associations(&icosahedron);
    }

    #[test]
    fn tetrahedron() {
        let scale: f32 = 1.0;
        let mesh = primitives::tetrahedron(scale);
        assert_congruent_nonboundary_mesh(&mesh);
        assert_face_associations(&mesh);
    }

    #[test]
    fn plane() {
        let scale: f32 = 1.0;
        let plane = primitives::plane(scale);
        assert_congruent_mesh(&plane);
        assert_face_associations(&plane);
    }

    #[test]
    fn triangle() {
        let scale: f32 = 1.0;
        let mesh = primitives::triangle(scale);
        assert_congruent_mesh(&mesh);
        assert_face_associations(&mesh);
    }

    #[test]
    fn subdivided_icosahedron() {
        let subdivisions: usize = 3;
        let planet_radius: f32 = 1.0;

        let icosahedron = primitives::icosahedron(planet_radius);
        let mesh = icosahedron.subdivide(subdivisions, &|a, b| {
                math::midpoint_arc(planet_radius, a, b)
            });
        assert_congruent_nonboundary_mesh(&mesh);
        assert_face_associations(&mesh);
    }

    #[test]
    fn subdivided_tetrahedron() {
        let subdivisions: usize = 3;
        let scale: f32 = 1.0;

        let tetrahedron = primitives::tetrahedron(scale);
        let mesh = tetrahedron.subdivide(subdivisions, &|a, b| {
                math::midpoint(a, b)
            });
        assert_congruent_nonboundary_mesh(&mesh);
        assert_face_associations(&mesh);
    }

    #[test]
    fn subdivided_triangle() {
        let subdivisions: usize = 3;
        let scale: f32 = 1.0;

        let tri = primitives::triangle(scale);
        let mesh = tri.subdivide(subdivisions, &|a, b| {
                math::midpoint(a, b)
            });
        assert_congruent_mesh(&mesh);
        assert_face_associations(&mesh);
    }

    #[test]
    fn subdivided_plane() {
        let subdivisions: usize = 3;
        let scale: f32 = 1.0;

        let plane = primitives::plane(scale);
        let mesh = plane.subdivide(subdivisions, &|a, b| {
                math::midpoint(a, b)
            });
        assert_congruent_mesh(&mesh);
        assert_face_associations(&mesh);
    }

    #[test]
    fn dual_of_icosahedron() {
        let scale: f32 = 1.0;
        let mesh = primitives::icosahedron(scale).generate_dual();
        assert_face_associations(&mesh);
    }
}
