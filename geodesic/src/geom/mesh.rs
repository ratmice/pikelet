
use super::*;

/// Mesh
///
/// The central bucket of attributes and connectivity information
///
#[derive(Clone, Debug)]
pub struct Mesh {
    /// Points in Spaaaaaaacccceeee!
    pub positions: Vec<Position>,

    /// Faces
    pub faces: Vec<Face>,

    /// Edges
    pub edges: Vec<Edge>,
}

impl Mesh {
    /// Create a new empty `Mesh`
    pub fn empty() -> Mesh {
        Mesh {
            positions: Vec::new(),
            faces: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Create a new empty `Mesh` with the speified capacity
    pub fn with_capacity(capacity: usize) -> Mesh {
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
        let p0 = self.positions[edge.position];
        let p1 = self.positions[self.edges[edge.next].position];
        midpoint_fn(p0, p1)
    }

    pub fn next_face_id(&self) -> FaceIndex {
        self.faces.len()
    }

    pub fn next_edge_id(&self) -> EdgeIndex {
        self.edges.len()
    }

    pub fn next_position_id(&self) -> PositionIndex {
        self.positions.len()
    }

    pub fn add_position(&mut self, p: Position) -> PositionIndex {
        let id = self.next_position_id();
        self.positions.push(p);
        id
    }

    pub fn add_boundary_edge(&mut self, pos: PositionIndex, face: FaceIndex, next: EdgeIndex) -> EdgeIndex {
        let id = self.next_edge_id();
        self.edges.push(Edge::new_boundary(pos, face, next));
        id
    }

    pub fn add_edge(&mut self, pos: PositionIndex, face: FaceIndex, next: EdgeIndex, adjacent: EdgeIndex) -> EdgeIndex {
        let id = self.next_edge_id();
        self.edges.push(Edge::new(pos, face, next, adjacent));
        id
    }

    pub fn make_adjacent(&mut self, a: EdgeIndex, b: EdgeIndex) {
        self.edges[a].adjacent = Some(b);
        self.edges[b].adjacent = Some(a);

        debug_assert!({
            let ref e0 = self.edges[a];
            let e0p0 = e0.position.clone();
            let e0p1 = self.edges[e0.next].position.clone();

            let ref e1 = self.edges[b];
            let e1p0 = e1.position.clone();
            let e1p1 = self.edges[e1.next].position.clone();

            e0p0 == e1p1 && e0p1 == e1p0
        });
    }

    /// Add three new `Edge`s and a `Face` to a `Mesh`.
    ///
    /// # Note
    ///
    /// Edges will not include adjacency information
    ///
    pub fn add_triangle(&mut self, p0: PositionIndex, p1: PositionIndex, p2: PositionIndex) -> FaceIndex {
        let id = self.next_face_id();
        let e0 = self.next_edge_id();
        let e1 = e0 + 1;
        let e2 = e1 + 1;

        self.add_boundary_edge(p0, id, e1);
        self.add_boundary_edge(p1, id, e2);
        self.add_boundary_edge(p2, id, e0);

        self.faces.push(Face::new(e0));

        id
    }
}
