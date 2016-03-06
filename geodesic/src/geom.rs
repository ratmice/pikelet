use cgmath::Point3;

use index::{self, ElementIndex};
use math;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)] pub struct NodeIndex(pub usize);
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)] pub struct EdgeIndex(pub usize);
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)] pub struct FaceIndex(pub usize);

impl ElementIndex for NodeIndex { type Element = Node; fn index(self) -> usize { self.0 } }
impl ElementIndex for EdgeIndex { type Element = Edge; fn index(self) -> usize { self.0 } }
impl ElementIndex for FaceIndex { type Element = Face; fn index(self) -> usize { self.0 } }

#[derive(Clone, Debug)]
pub struct Node {
    pub position: Point3<f32>,
    pub edges: Vec<EdgeIndex>,
    pub faces: Vec<FaceIndex>,

    // Pentagon
    // pub edges: [EdgeIndex; 5],
    // pub faces: [FaceIndex; 5],

    // Hexagon
    // pub edges: [EdgeIndex; 6],
    // pub faces: [FaceIndex; 6],
}

#[derive(Clone, Debug)]
pub struct Edge {
    pub nodes: [NodeIndex; 2],
    pub faces: Vec<FaceIndex>,
    // pub faces: [EdgeIndex; 2],
}

#[derive(Clone, Debug)]
pub struct Face {
    pub nodes: [NodeIndex; 3],
    pub edges: Vec<EdgeIndex>,
    // pub edges: [EdgeIndex; 3],
}

#[derive(Clone, Debug)]
pub struct Geometry {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub faces: Vec<Face>,
}

impl Geometry {
    pub fn subdivide(&self, radius: f32, count: usize) -> Geometry {
        (0..count).fold(self.clone(), |acc, _| acc.subdivide_once(radius))
    }

    pub fn subdivide_once(&self, radius: f32) -> Geometry {
        let mut nodes = Vec::with_capacity(self.nodes.len() * 2);
        let mut edges = Vec::with_capacity(self.edges.len() * 2);
        let mut faces = Vec::with_capacity(self.faces.len() * 4);

        let push_node = |nodes: &mut Vec<_>, node| {
            nodes.push(node);
            NodeIndex(nodes.len() - 1)
        };

        let push_edge = |edges: &mut Vec<_>, edge| {
            edges.push(edge);
            EdgeIndex(edges.len() - 1)
        };

        for face in &self.faces {
            //
            //          n0
            //          /\
            //         /  \
            //    n5  /____\  n3
            //       /\    /\
            //      /  \  /  \
            //     /____\/____\
            //   n2     n4     n1
            //

            let p0 = math::set_radius(index::get(&self.nodes, face.nodes[0]).position, radius);
            let p1 = math::set_radius(index::get(&self.nodes, face.nodes[1]).position, radius);
            let p2 = math::set_radius(index::get(&self.nodes, face.nodes[2]).position, radius);
            let p3 = math::set_radius(math::midpoint(p0, p1), radius);
            let p4 = math::set_radius(math::midpoint(p1, p2), radius);
            let p5 = math::set_radius(math::midpoint(p2, p0), radius);

            let n0 = push_node(&mut nodes, Node { position: p0, edges: vec![], faces: vec![] });
            let n1 = push_node(&mut nodes, Node { position: p1, edges: vec![], faces: vec![] });
            let n2 = push_node(&mut nodes, Node { position: p2, edges: vec![], faces: vec![] });
            let n3 = push_node(&mut nodes, Node { position: p3, edges: vec![], faces: vec![] });
            let n4 = push_node(&mut nodes, Node { position: p4, edges: vec![], faces: vec![] });
            let n5 = push_node(&mut nodes, Node { position: p5, edges: vec![], faces: vec![] });

            let n0_n3 = push_edge(&mut edges, Edge { nodes: [n0, n3], faces: vec![] });
            let n0_n5 = push_edge(&mut edges, Edge { nodes: [n0, n5], faces: vec![] });
            let n5_n3 = push_edge(&mut edges, Edge { nodes: [n5, n3], faces: vec![] });
            let n5_n4 = push_edge(&mut edges, Edge { nodes: [n5, n4], faces: vec![] });
            let n5_n2 = push_edge(&mut edges, Edge { nodes: [n5, n2], faces: vec![] });
            let n2_n4 = push_edge(&mut edges, Edge { nodes: [n2, n4], faces: vec![] });
            let n3_n1 = push_edge(&mut edges, Edge { nodes: [n3, n1], faces: vec![] });
            let n3_n4 = push_edge(&mut edges, Edge { nodes: [n3, n4], faces: vec![] });
            let n4_n1 = push_edge(&mut edges, Edge { nodes: [n4, n1], faces: vec![] });

            faces.push(Face { nodes: [n0, n3, n5], edges: vec![n0_n3, n0_n5, n5_n3] });
            faces.push(Face { nodes: [n3, n1, n4], edges: vec![n3_n1, n3_n4, n4_n1] });
            faces.push(Face { nodes: [n5, n4, n2], edges: vec![n5_n4, n5_n2, n2_n4] });
            faces.push(Face { nodes: [n5, n3, n4], edges: vec![n5_n3, n5_n4, n3_n4] });
        }

        Geometry {
            nodes: nodes,
            edges: edges,
            faces: faces,
        }.add_indices_from_faces()
    }

    fn add_indices_from_faces(mut self) -> Geometry {
        for (i, edge) in self.edges.iter().enumerate() {
            index::get_mut(&mut self.nodes, edge.nodes[0]).edges.push(EdgeIndex(i));
            index::get_mut(&mut self.nodes, edge.nodes[1]).edges.push(EdgeIndex(i));
        }

        for (i, face) in self.faces.iter().enumerate() {
            index::get_mut(&mut self.nodes, face.nodes[0]).faces.push(FaceIndex(i));
            index::get_mut(&mut self.nodes, face.nodes[1]).faces.push(FaceIndex(i));
            index::get_mut(&mut self.nodes, face.nodes[2]).faces.push(FaceIndex(i));
        }

        for (i, face) in self.faces.iter().enumerate() {
            index::get_mut(&mut self.edges, face.edges[0]).faces.push(FaceIndex(i));
            index::get_mut(&mut self.edges, face.edges[1]).faces.push(FaceIndex(i));
            index::get_mut(&mut self.edges, face.edges[2]).faces.push(FaceIndex(i));
        }

        self
    }
}

/// The base geometry of a [regular iocosahedron](https://en.wikipedia.org/wiki/Regular_icosahedron).
pub fn icosahedron() -> Geometry {
    use self::EdgeIndex as E;
    use self::NodeIndex as N;

    // The coordinates of the iocosahedron are are described by the
    // cyclic permutations of (±ϕ, ±1, 0), where ϕ is the [Golden Ratio]
    // (https://en.wikipedia.org/wiki/Golden_ratio).
    let phi = (1.0 + f32::sqrt(5.0)) / 2.0;
    let nodes = vec![
        Node { position: math::set_radius(Point3::new( phi,  1.0,  0.0), 1.0), edges: vec![], faces: vec![] },
        Node { position: math::set_radius(Point3::new( phi, -1.0,  0.0), 1.0), edges: vec![], faces: vec![] },
        Node { position: math::set_radius(Point3::new(-phi,  1.0,  0.0), 1.0), edges: vec![], faces: vec![] },
        Node { position: math::set_radius(Point3::new(-phi, -1.0,  0.0), 1.0), edges: vec![], faces: vec![] },
        Node { position: math::set_radius(Point3::new( 0.0,  phi,  1.0), 1.0), edges: vec![], faces: vec![] },
        Node { position: math::set_radius(Point3::new( 0.0,  phi, -1.0), 1.0), edges: vec![], faces: vec![] },
        Node { position: math::set_radius(Point3::new( 0.0, -phi,  1.0), 1.0), edges: vec![], faces: vec![] },
        Node { position: math::set_radius(Point3::new( 0.0, -phi, -1.0), 1.0), edges: vec![], faces: vec![] },
        Node { position: math::set_radius(Point3::new( 1.0,  0.0,  phi), 1.0), edges: vec![], faces: vec![] },
        Node { position: math::set_radius(Point3::new(-1.0,  0.0,  phi), 1.0), edges: vec![], faces: vec![] },
        Node { position: math::set_radius(Point3::new( 1.0,  0.0, -phi), 1.0), edges: vec![], faces: vec![] },
        Node { position: math::set_radius(Point3::new(-1.0,  0.0, -phi), 1.0), edges: vec![], faces: vec![] },
    ];

    let edges = vec![
        Edge { nodes: [N( 0), N( 1)], faces: vec![] },
        Edge { nodes: [N( 0), N( 4)], faces: vec![] },
        Edge { nodes: [N( 0), N( 5)], faces: vec![] },
        Edge { nodes: [N( 0), N( 8)], faces: vec![] },
        Edge { nodes: [N( 0), N(10)], faces: vec![] },
        Edge { nodes: [N( 1), N( 6)], faces: vec![] },
        Edge { nodes: [N( 1), N( 7)], faces: vec![] },
        Edge { nodes: [N( 1), N( 8)], faces: vec![] },
        Edge { nodes: [N( 1), N(10)], faces: vec![] },
        Edge { nodes: [N( 2), N( 3)], faces: vec![] },
        Edge { nodes: [N( 2), N( 4)], faces: vec![] },
        Edge { nodes: [N( 2), N( 5)], faces: vec![] },
        Edge { nodes: [N( 2), N( 9)], faces: vec![] },
        Edge { nodes: [N( 2), N(11)], faces: vec![] },
        Edge { nodes: [N( 3), N( 6)], faces: vec![] },
        Edge { nodes: [N( 3), N( 7)], faces: vec![] },
        Edge { nodes: [N( 3), N( 9)], faces: vec![] },
        Edge { nodes: [N( 3), N(11)], faces: vec![] },
        Edge { nodes: [N( 4), N( 5)], faces: vec![] },
        Edge { nodes: [N( 4), N( 8)], faces: vec![] },
        Edge { nodes: [N( 4), N( 9)], faces: vec![] },
        Edge { nodes: [N( 5), N(10)], faces: vec![] },
        Edge { nodes: [N( 5), N(11)], faces: vec![] },
        Edge { nodes: [N( 6), N( 7)], faces: vec![] },
        Edge { nodes: [N( 6), N( 8)], faces: vec![] },
        Edge { nodes: [N( 6), N( 9)], faces: vec![] },
        Edge { nodes: [N( 7), N(10)], faces: vec![] },
        Edge { nodes: [N( 7), N(11)], faces: vec![] },
        Edge { nodes: [N( 8), N( 9)], faces: vec![] },
        Edge { nodes: [N(10), N(11)], faces: vec![] },
    ];

    let faces = vec![
        Face { nodes: [N( 8), N( 1), N( 0)], edges: vec![E( 0), E( 7), E( 3)] },
        Face { nodes: [N( 5), N( 4), N( 0)], edges: vec![E( 1), E(18), E( 2)] },
        Face { nodes: [N(10), N( 5), N( 0)], edges: vec![E( 2), E(21), E( 4)] },
        Face { nodes: [N( 4), N( 8), N( 0)], edges: vec![E( 3), E(19), E( 1)] },
        Face { nodes: [N( 1), N(10), N( 0)], edges: vec![E( 4), E( 8), E( 0)] },
        Face { nodes: [N( 8), N( 6), N( 1)], edges: vec![E( 5), E(24), E( 7)] },
        Face { nodes: [N( 6), N( 7), N( 1)], edges: vec![E( 6), E(23), E( 5)] },
        Face { nodes: [N( 7), N(10), N( 1)], edges: vec![E( 8), E(26), E( 6)] },
        Face { nodes: [N(11), N( 3), N( 2)], edges: vec![E( 9), E(17), E(13)] },
        Face { nodes: [N( 9), N( 4), N( 2)], edges: vec![E(10), E(20), E(12)] },
        Face { nodes: [N( 4), N( 5), N( 2)], edges: vec![E(11), E(18), E(10)] },
        Face { nodes: [N( 3), N( 9), N( 2)], edges: vec![E(12), E(16), E( 9)] },
        Face { nodes: [N( 5), N(11), N( 2)], edges: vec![E(13), E(22), E(11)] },
        Face { nodes: [N( 7), N( 6), N( 3)], edges: vec![E(14), E(23), E(15)] },
        Face { nodes: [N(11), N( 7), N( 3)], edges: vec![E(15), E(27), E(17)] },
        Face { nodes: [N( 6), N( 9), N( 3)], edges: vec![E(16), E(25), E(14)] },
        Face { nodes: [N( 9), N( 8), N( 4)], edges: vec![E(19), E(28), E(20)] },
        Face { nodes: [N(10), N(11), N( 5)], edges: vec![E(22), E(29), E(21)] },
        Face { nodes: [N( 8), N( 9), N( 6)], edges: vec![E(25), E(28), E(24)] },
        Face { nodes: [N(11), N(10), N( 7)], edges: vec![E(26), E(29), E(27)] },
    ];

    Geometry {
        nodes: nodes,
        edges: edges,
        faces: faces,
    }.add_indices_from_faces()
}
