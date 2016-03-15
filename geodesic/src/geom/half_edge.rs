use std::collections::HashMap;
use cgmath::{Vector4, Point3};
use math;

///////////////////////////////////////////////////////////////////////////////
// Some basic type aliases in order to attemp self-documentation

pub type Index = usize;
pub type Position = Point3<f32>;
pub type Normal = Point3<f32>;
pub type Color = Vector4<f32>;

///////////////////////////////////////////////////////////////////////////////
// Principle mesh attribute index struct
// TODO: Attributes need to be arbitrary.
//       Like a hashmap from "name" -> Vec<T>.
//
//       Some examples:
//         - P -> Vec<Position>
//         - C -> Vec<Color>
//         - N -> Vec<Point3>
//         - scatter -> Vec<f32>
//         - elevation -> Vec<f32>
//         - inclination -> Vec<f32>
//
//       Obviously some of those are principle
//       types used to draw stuff, but others
//       would be used in procgen tasks perhaps.
//       This is a common trick in apps like Houdini.
//
#[derive(Clone, Debug)]
pub struct AttributeIndex {
    pub position: Index,        // Positions are required.
}

impl AttributeIndex {
    pub fn new(position: Index) -> AttributeIndex {
        AttributeIndex {
            position: position
        }
    }
}

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
    pub root: Index,
}

impl Face {
    pub fn new(root: Index) -> Face {
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
    pub attributes: AttributeIndex,

    // The face that this edge is associated with.
    pub face: Index,

    // The index of the next edge/vert around the face.
    pub next: Index,

    // Oppositely oriented adjacent HalfEdge.
    // If this is None then we have a boundary edge.
    pub adjacent: Option<Index>,
}

impl HalfEdge {
    pub fn new(point: Index, face: Index, next: Index, adjacent: Index) -> HalfEdge {
        HalfEdge {
            attributes: AttributeIndex::new(point),
            face: face,
            next: next,
            adjacent: Some(adjacent)
        }
    }

    pub fn new_boundary(point: Index, face: Index, next: Index) -> HalfEdge {
        HalfEdge {
            attributes: AttributeIndex::new(point),
            face: face,
            next: next,
            adjacent: None
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
    pub vertices: Vec<HalfEdge>
}

impl Mesh {

    // Return the Position of the midpoint along an edge
    fn midpoint(&self, radius: f32, e0: Index, e1: Index, vertices: &Vec<HalfEdge>, positions: &Vec<Position>) -> Position {
        let ref p0 = positions[vertices[e0].attributes.position];
        let ref p1 = positions[vertices[e1].attributes.position];

        //math::set_radius(math::midpoint(p0, p1), radius)
        math::midpoint(p0, p1)
    }
    
    pub fn subdivide(&self, radius: f32, count: usize) -> Mesh {
        (0..count).fold(self.clone(), |acc, _| acc.subdivide_once(radius))
    }

    // - For each face, get the edge loop, split each each edge.
    // - Once each edge is split you make 3 new faces.
    // - Always split adjacent edges in order to maintain correct indexing.
    //
    // Keep track of any attributes so that we don't duplicate them.
    //
    // NOTE: We're baking in the idea that a mesh *only* ever contains
    //       triangles. That's fine of course, but if any other functions
    //       potentially generate faces with more than 3 edges you'll
    //       need to triangulate the Mesh first.
    //
    // NOTE: The method of subdivision is illustrated below:
    //
    //          v0         |  v0 ____v3____ v2
    //          /\         |    \    /\    /
    //         /  \        |     \  /  \  /
    //    v3  /____\  v5   |   v4 \/____\/ v5
    //       /\    /\      |       \    /
    //      /  \  /  \     |        \  /
    //     /____\/____\    |         \/
    //   v1     v4     v2  |         v1
    //
    pub fn subdivide_once(&self, radius: f32) -> Mesh {
        let mut visited_edges: HashMap<Index, (Index, Index)> = HashMap::new();
        let mut new_positions: HashMap<Index, Index> = HashMap::new();
        
        let mut positions = self.positions.clone();
        let mut vertices = self.vertices.clone();
        let mut faces = self.faces.clone();

        for (f0, face) in self.faces.iter().enumerate() {
            let e0 = face.root.clone();
            let e1 = self.vertices[e0].next.clone();
            
            let p = self.midpoint(radius, e0, e1, &self.vertices, &self.positions);
            let p_index = positions.len();
            println!("New position: {} / {}, {}, {}", p_index, p.x, p.y, p.z);
            positions.push(p);
            
            let new_edge = HalfEdge::new_boundary(p_index, vertices[e0].face.clone(), e1);
            vertices[e0].next = vertices.len();
            vertices.push(new_edge);
        }

        assert!(positions.len() > self.positions.len());

        Mesh {
            positions: positions,
            faces: faces,
            vertices: vertices
        }
    }
}

pub fn triangle(scale: f32) -> Mesh {
    let extent = scale / 2.0;
    let positions = vec![
        Point3::new(0.0, extent, 0.0),
        Point3::new(-extent, -extent, 0.0),
        Point3::new(extent, -extent, 0.0)
    ];

    let faces = vec![
        Face::new(0)
    ];

    let vertices = vec![
        HalfEdge::new_boundary(0, 0, 1),
        HalfEdge::new_boundary(1, 0, 2),
        HalfEdge::new_boundary(2, 0, 0),
    ];

    Mesh {
        positions: positions,
        vertices: vertices,
        faces: faces
    }
}

pub fn icosahedron(radius: f32) -> Mesh {
    // The coordinates of the iocosahedron are are described by the
    // cyclic permutations of (±ϕ, ±1, 0), where ϕ is the [Golden Ratio]
    // (https://en.wikipedia.org/wiki/Golden_ratio).
    // NOTE: The order here is such that 0 - 11 line up with the logical
    //       winding of faces over the "net" of the polyhedron.
    //       (would be cool if you could annotate source code with images
    //        to explain notes like this!)
    let phi = (1.0 + f32::sqrt(5.0)) / 2.0;
    let positions = vec![
        math::set_radius(Point3::new( 0.0,  phi, -1.0), radius),
        math::set_radius(Point3::new(-phi,  1.0,  0.0), radius),
        math::set_radius(Point3::new( 0.0,  phi,  1.0), radius),
        math::set_radius(Point3::new( phi,  1.0,  0.0), radius),
        math::set_radius(Point3::new( 1.0,  0.0, -phi), radius),
        math::set_radius(Point3::new(-1.0,  0.0, -phi), radius),
        math::set_radius(Point3::new(-1.0,  0.0,  phi), radius),
        math::set_radius(Point3::new( 1.0,  0.0,  phi), radius),
        math::set_radius(Point3::new( phi, -1.0,  0.0), radius),
        math::set_radius(Point3::new( 0.0, -phi, -1.0), radius),
        math::set_radius(Point3::new(-phi, -1.0,  0.0), radius),
        math::set_radius(Point3::new( 0.0, -phi,  1.0), radius),
    ];

                       // Edges around
    let faces = vec![  // the face:
        Face::new( 0), //  0,  1,  2
        Face::new( 3), //  3,  4,  5
        Face::new( 6), //  6,  7,  8
        Face::new( 9), //  9, 10, 11
        Face::new(12), // 12, 13, 14
        Face::new(15), // 15, 16, 17
        Face::new(18), // 18, 19, 20
        Face::new(21), // 21, 22, 23
        Face::new(24), // 24, 25, 26
        Face::new(27), // 27, 28, 29
        Face::new(30), // 30, 31, 32
        Face::new(33), // 33, 34, 35
        Face::new(36), // 36, 37, 38
        Face::new(39), // 39, 40, 41
        Face::new(42), // 42, 43, 44
        Face::new(45), // 45, 46, 47
        Face::new(48), // 48, 49, 50
        Face::new(51), // 51, 52, 53
        Face::new(54), // 54, 55, 56
        Face::new(57), // 57, 58, 59
    ];
    
    let vertices = vec![
        // Face 0     // point, face, next, adj
        HalfEdge::new(       0,    0,    1,  14),
        HalfEdge::new(       1,    0,    2,  17),
        HalfEdge::new(       2,    0,    0,   3),
        // Face 1
        HalfEdge::new(       0,    1,    4,   2),
        HalfEdge::new(       2,    1,    5,  23),
        HalfEdge::new(       3,    1,    3,   6),
        // Face 2
        HalfEdge::new(       0,    2,    7,   5),
        HalfEdge::new(       3,    2,    8,  29),
        HalfEdge::new(       4,    2,    6,   9),
        // Face 3
        HalfEdge::new(       0,    3,   10,   8),
        HalfEdge::new(       4,    3,   11,  35),
        HalfEdge::new(       5,    3,    9,  12),
        // Face 4
        HalfEdge::new(       0,    4,   13,  11),
        HalfEdge::new(       5,    4,   14,  41),
        HalfEdge::new(       1,    4,   12,   0),
        // Face 5
        HalfEdge::new(       1,    5,   16,  44),
        HalfEdge::new(       6,    5,   17,  18),
        HalfEdge::new(       2,    5,   15,   1),
        // Face 6
        HalfEdge::new(       2,    6,   19,  16),
        HalfEdge::new(       7,    6,   20,  47),
        HalfEdge::new(       3,    6,   18,  21),
        // Face 7
        HalfEdge::new(       3,    7,   22,  20),
        HalfEdge::new(       8,    7,   23,  24),
        HalfEdge::new(       4,    7,   21,   4),
        // Face 8
        HalfEdge::new(       4,    8,   25,  22),
        HalfEdge::new(       9,    8,   26,  50),
        HalfEdge::new(       5,    8,   24,  27),
        // Face 9
        HalfEdge::new(       5,    9,   28,  26),
        HalfEdge::new(      10,    9,   29,  30),
        HalfEdge::new(       1,    9,   27,   7),
        // Face 10
        HalfEdge::new(       2,   10,   31,  29),
        HalfEdge::new(       6,   10,   32,  53),
        HalfEdge::new(       7,   10,   30,  33),
        // Face 11
        HalfEdge::new(       3,   11,   34,  32),
        HalfEdge::new(       7,   11,   35,  36),
        HalfEdge::new(       8,   11,   33,  10),
        // Face 12
        HalfEdge::new(       4,   12,   37,  34),
        HalfEdge::new(       8,   12,   38,  56),
        HalfEdge::new(       9,   12,   36,  39),
        // Face 13
        HalfEdge::new(       5,   13,   40,  38),
        HalfEdge::new(       9,   13,   41,  42),
        HalfEdge::new(      10,   13,   39,  13),
        // Face 14
        HalfEdge::new(       1,   14,   43,  40),
        HalfEdge::new(      10,   14,   44,  59),
        HalfEdge::new(       6,   14,   42,  15),
        // Face 15
        HalfEdge::new(       6,   15,   46,  58),
        HalfEdge::new(      11,   15,   47,  48),
        HalfEdge::new(       7,   15,   45,  19),
        // Face 16
        HalfEdge::new(       7,   16,   49,  46),
        HalfEdge::new(      11,   16,   50,  51),
        HalfEdge::new(       8,   16,   48,  25),
        // Face 17
        HalfEdge::new(       8,   17,   52,  49),
        HalfEdge::new(      11,   17,   53,  54),
        HalfEdge::new(       9,   17,   51,  31),
        // Face 18
        HalfEdge::new(       9,   18,   55,  52),
        HalfEdge::new(      11,   18,   56,  57),
        HalfEdge::new(      10,   18,   54,  37),
        // Face 19
        HalfEdge::new(      10,   19,   58,  55),
        HalfEdge::new(      11,   19,   59,  45),
        HalfEdge::new(       6,   19,   57,  43),
    ];

    Mesh {
        positions: positions,
        faces: faces,
        vertices: vertices
    }
}
