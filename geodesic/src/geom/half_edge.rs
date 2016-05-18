use std::collections::HashMap;
use cgmath::Point3;

///////////////////////////////////////////////////////////////////////////////
// Some basic type aliases in order to attemp self-documentation

pub type EdgeIndex = usize;
pub type PositionIndex = usize;
pub type FaceIndex = usize;
pub type Position = Point3<f32>;

///////////////////////////////////////////////////////////////////////////////
// The Face
//
// TODO: Is the face really so sparse?
//       Probably not! Because there is a bunch of attributes, seeds, values,
//       parameters, references to things, and so on, and so on; that could
//       be associated and organized with a single Face. So let's assume
//       that connectivity aside, we'll be stuffing stuff into the Face struct
//       eventually.
//
#[derive(Clone, Debug)]
pub struct Face {
    // The index of the first edge to define this face.
    pub root: EdgeIndex,
}

impl Face {
    pub fn new(root: EdgeIndex) -> Face {
        Face { root: root }
    }
}

///////////////////////////////////////////////////////////////////////////////
// Our primary entity for navigating the topology of a Mesh
//
// Vertices and edges are essentially the same in this data structure
// So I've deviated a bit from the vernacular and "collapsed" the
// Vertex and HalfEdge structures into a single struct.
//
#[derive(Clone, Debug)]
pub struct HalfEdge {
    // Attribute index for this vertex.
    pub position: PositionIndex,

    // The face that this edge is associated with.
    pub face: FaceIndex,

    // The index of the next edge/vert around the face.
    pub next: EdgeIndex,

    // Oppositely oriented adjacent HalfEdge.
    // If this is None then we have a boundary edge.
    pub adjacent: Option<EdgeIndex>,
}

impl HalfEdge {
    pub fn new(point: PositionIndex, face: FaceIndex, next: EdgeIndex, adjacent: EdgeIndex) -> HalfEdge {
        HalfEdge {
            position: point,
            face: face,
            next: next,
            adjacent: Some(adjacent),
        }
    }

    pub fn new_boundary(point: PositionIndex, face: FaceIndex, next: EdgeIndex) -> HalfEdge {
        HalfEdge {
            position: point,
            face: face,
            next: next,
            adjacent: None,
        }
    }

    pub fn is_boundary(&self) -> bool {
        self.adjacent.is_none()
    }
}

///////////////////////////////////////////////////////////////////////////////
// The central bucket of attributes and connectivity information
//
#[derive(Clone, Debug)]
pub struct Mesh {
    // Attributes
    pub positions: Vec<Position>,

    // Connectivity information
    pub faces: Vec<Face>,
    pub edges: Vec<HalfEdge>,
}

impl Mesh {
    pub fn subdivide<F>(&self, count: usize, midpoint_fn: &F) -> Mesh
        where F: Fn(Position, Position) -> Position
    {
        (0..count).fold(self.clone(), |acc, level| {
            println!("Performing subdivision {}/{}", level+1, count);
            acc.subdivide_once(&midpoint_fn)
        })
    }

    // NOTE: The method of subdivision is illustrated below:
    //
    //          v0         |  v0 ____v5____ v2
    //          /\         |    \    /\    /
    //         /  \        |     \  /  \  /
    //    v3  /____\  v5   |   v3 \/____\/ v4
    //       /\    /\      |       \    /
    //      /  \  /  \     |        \  /
    //     /____\/____\    |         \/
    //   v1     v4     v2  |         v1
    //
    pub fn subdivide_once<F>(&self, midpoint_fn: &F) -> Mesh
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

            new_positions.insert(index, mp_index.clone());
            // Uncomment to break stuff
            // match edge.adjacent {
            //     Some(adjacent_index) => {
            //         new_positions.insert(adjacent_index.clone(), mp_index.clone());
            //     }
            //     None => ()
            // }
        }

        // Create our new faces
        for face in self.faces.iter() {
            let in_e0 = face.root.clone();
            let in_e1 = self.edges[in_e0].next.clone();
            let in_e2 = self.edges[in_e1].next.clone();

            assert!(self.edges[in_e2].next == in_e0);

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
            let p0 = self.edges[in_e0].position.clone();
            let p1 = self.edges[in_e1].position.clone();
            let p2 = self.edges[in_e2].position.clone();

            // Midpoint position indices
            let p3 = new_positions.get(&in_e0).unwrap().clone();
            let p4 = new_positions.get(&in_e1).unwrap().clone();
            let p5 = new_positions.get(&in_e2).unwrap().clone();

            faces.push(make_face(f0, e0,  e1,  e2, p0, p3, p5, &mut edges)); // face 0
            faces.push(make_face(f1, e3,  e4,  e5, p3, p1, p4, &mut edges)); // face 1
            faces.push(make_face(f2, e6,  e7,  e8, p3, p4, p5, &mut edges)); // face 2
            faces.push(make_face(f3, e9, e10, e11, p5, p4, p2, &mut edges)); // face 3
            

            split_edges.insert(in_e0.clone(), (e0.clone(), e3.clone()));
            split_edges.insert(in_e1.clone(), (e4.clone(), e10.clone()));
            split_edges.insert(in_e2.clone(), (e11.clone(), e2.clone()));
            

            ///////////////////////////////////////////////////////////////
            // Setup adjacency for internal edges (info we already have)

            // Face 0
            // edges[e0]
            edges[e1].adjacent = Some(e6.clone());
            // edges[e2]
            
            // Face 1
            // edges[e3]
            // edges[e4]
            edges[e5].adjacent = Some(e7.clone());
            
            // Face 2
            edges[e6].adjacent = Some(e1.clone());
            edges[e7].adjacent = Some(e5.clone());
            edges[e8].adjacent = Some(e9.clone());

            // Face 3
            edges[e9].adjacent = Some(e8.clone());
            // edges[e10]
            // edges[e11]
        }

        assert!(split_edges.len() == self.edges.len());
        
        // Update adjacency for remaining edges
        for (index, &(a, b)) in split_edges.iter() {
            let adjacent_edge = self.edges[*index].adjacent.unwrap().clone();
            let &(b_adjacent, a_adjacent) = split_edges.get(&adjacent_edge).unwrap();
            edges[a].adjacent = Some(a_adjacent);
            edges[b].adjacent = Some(b_adjacent);
        }

        Mesh {
            positions: positions,
            faces: faces,
            edges: edges
        }
    }

    fn edge_midpoint<F>(&self, edge: &HalfEdge, midpoint_fn: &F) -> Position
        where F: Fn(Position, Position) -> Position
    {
        let p0 = self.positions[edge.position];
        let p1 = self.positions[self.edges[edge.next].position];
        midpoint_fn(p0, p1)
    }
}

fn make_face(f: FaceIndex, e0: EdgeIndex, e1: EdgeIndex, e2: EdgeIndex,
             p0: PositionIndex, p1: PositionIndex, p2: PositionIndex,
             edges: &mut Vec<HalfEdge>) -> Face
{
    //println!("Making face for {} -> {} -> {}", e0, e1, e2);
    edges.push(HalfEdge::new_boundary(p0.clone(), f.clone(), e1.clone()));
    assert!(edges.len() == e0+1);
    edges.push(HalfEdge::new_boundary(p1.clone(), f.clone(), e2.clone()));
    assert!(edges.len() == e1+1);
    edges.push(HalfEdge::new_boundary(p2.clone(), f.clone(), e0.clone()));
    assert!(edges.len() == e2+1);

    Face::new(e0)
}
