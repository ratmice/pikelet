use std::slice::Iter as SliceIter;

use super::*;

/// Mesh
///
/// The central bucket of attributes and connectivity information
///
#[derive(Clone, Debug)]
pub struct Mesh<Position> {
    /// Points in Spaaaaaaacccceeee!
    pub positions: Vec<Position>,

    /// Faces
    pub faces: Vec<Face>,

    /// Edges
    pub edges: Vec<Edge>,
}

impl<Position: Copy> Mesh<Position> {
    /// Create a new empty `Mesh`
    pub fn empty() -> Mesh<Position> {
        Mesh {
            positions: Vec::new(),
            faces: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Create a new empty `Mesh` with the speified capacity
    pub fn with_capacity(capacity: usize) -> Mesh<Position> {
        Mesh {
            positions: Vec::with_capacity(capacity),
            faces: Vec::with_capacity(capacity),
            edges: Vec::with_capacity(capacity),
        }
    }

    /// Returns a new `geom::Position` using the provided function to calculate it.
    pub fn edge_midpoint<F>(&self, edge: &Edge, midpoint_fn: &F) -> Position
        where F: Fn(Position, Position) -> Position
    {
        let p0 = self.position(edge.position).unwrap();
        let p1 = self.position(self.edge(edge.next).unwrap().position).unwrap();
        midpoint_fn(*p0, *p1)
    }

    pub fn edge(&self, id: EdgeIndex) -> Option<&Edge> {
        self.edges.get(id.0)
    }

    pub fn edge_mut(&mut self, id: EdgeIndex) -> Option<&mut Edge> {
        self.edges.get_mut(id.0)
    }

    pub fn face(&self, id: FaceIndex) -> Option<&Face> {
        self.faces.get(id.0)
    }

    pub fn face_mut(&mut self, id: FaceIndex) -> Option<&mut Face> {
        self.faces.get_mut(id.0)
    }

    pub fn position(&self, id: PositionIndex) -> Option<&Position> {
        self.positions.get(id.0)
    }

    pub fn position_mut(&mut self, id: PositionIndex) -> Option<&mut Position> {
        self.positions.get_mut(id.0)
    }

    pub fn add_position(&mut self, p: Position) -> PositionIndex {
        let id = PositionIndex(self.positions.len());
        self.positions.push(p);
        id
    }

    pub fn add_boundary_edge(&mut self,
                             pos: PositionIndex,
                             face: FaceIndex,
                             next: EdgeIndex)
                             -> EdgeIndex {
        let id = EdgeIndex(self.edges.len());
        self.edges.push(Edge::new_boundary(pos, face, next));
        id
    }

    pub fn add_edge(&mut self,
                    pos: PositionIndex,
                    face: FaceIndex,
                    next: EdgeIndex,
                    adjacent: EdgeIndex)
                    -> EdgeIndex {
        let id = EdgeIndex(self.edges.len());
        self.edges.push(Edge::new(pos, face, next, adjacent));
        id
    }

    #[cfg_attr(feature = "cargo-clippy", allow(panic_params))]
    pub fn make_adjacent(&mut self, a: EdgeIndex, b: EdgeIndex) {
        self.edge_mut(a).unwrap().adjacent = Some(b);
        self.edge_mut(b).unwrap().adjacent = Some(a);

        debug_assert!({
                          let e0 = self.edge(a).unwrap();
                          let e0p0 = e0.position;
                          let e0p1 = self.edge(e0.next).unwrap().position;

                          let e1 = self.edge(b).unwrap();
                          let e1p0 = e1.position;
                          let e1p1 = self.edge(e1.next).unwrap().position;

                          e0p0 == e1p1 && e0p1 == e1p0
                      });
    }

    /// Add three new `Edge`s and a `Face` to a `Mesh`.
    ///
    /// # Note
    ///
    /// Edges will not include adjacency information
    ///
    pub fn add_triangle(&mut self,
                        p0: PositionIndex,
                        p1: PositionIndex,
                        p2: PositionIndex)
                        -> (FaceIndex, (EdgeIndex, EdgeIndex, EdgeIndex)) {
        let id = FaceIndex(self.faces.len());
        let e0 = EdgeIndex(self.edges.len());
        let e1 = EdgeIndex(e0.0 + 1);
        let e2 = EdgeIndex(e1.0 + 1);

        self.add_boundary_edge(p0, id, e1);
        self.add_boundary_edge(p1, id, e2);
        self.add_boundary_edge(p2, id, e0);

        self.faces.push(Face::new(e0));

        (id, (e0, e1, e2))
    }

    /// Iterate through the faces, yielding point indices as if each face
    /// were a triangle.
    pub fn triangles(&self) -> Triangles<Position> {
        Triangles {
            mesh: self,
            iter: self.faces.iter(),
        }
    }
}

pub struct Triangles<'a, Position: 'a> {
    mesh: &'a Mesh<Position>,
    iter: SliceIter<'a, Face>,
}

impl<'a, Position: Copy> Iterator for Triangles<'a, Position> {
    type Item = [PositionIndex; 3];

    fn next(&mut self) -> Option<[PositionIndex; 3]> {
        self.iter
            .next()
            .map(|face| {
                     let e0 = self.mesh.edge(face.root).unwrap();
                     let e1 = self.mesh.edge(e0.next).unwrap();
                     let e2 = self.mesh.edge(e1.next).unwrap();

                     [e0.position, e1.position, e2.position]
                 })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}
