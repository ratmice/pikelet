
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
pub struct AttributeIndex {
    pub position: Index,        // Positions are required.
    pub color: Option<Index>,   // If a mesh has vertex colors.
    pub normal: Option<Index>,  // If a mesh has normals.
}

impl AttributeIndex {
    pub fn new(position: Index) -> AttributeIndex {
        AttributeIndex {
            position: position,
            color: None,
            normal: None
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// Vertices are an attribute index and an edge reference
//
pub struct Vertex {
    // Attribute index for this vertex.
    pub attributes: AttributeIndex,

    // The HalfEdge eminating from this vertex.
    pub edge: Index,
}

impl Vertex {
    pub fn new(edge: Index, attributes: AttributeIndex) -> Vertex {
        Vertex {
            attributes: attributes,
            edge: edge
        }
    }

    pub fn has_color(&self) -> bool {
        self.attributes.color.is_some()
    }

    pub fn has_normal(&self) -> bool {
        self.attributes.normal.is_some()
    }
}

///////////////////////////////////////////////////////////////////////////////
// Our primary entity for navigating the topology of a Mesh
//
pub struct HalfEdge {
    // Vertex that this half-edge points toward.
    pub vertex: Index,

    // The face that this edge borders.
    pub face: Index,

    // The next HalfEdge around the face.
    pub next: Index,

    // Oppositely oriented adjacent HalfEdge.
    // If this is None then we have a boundary edge.
    pub adjacent: Option<Index>,
}

impl HalfEdge {
    pub fn new(vertex: Index, face: Index, next: Index, adjacent: Index) -> HalfEdge {
        HalfEdge {
            vertex: vertex,
            face: face,
            next: next,
            adjacent: Some(adjacent)
        }
    }

    pub fn new_boundary_edge(vertex: Index, face: Index, next: Index) -> HalfEdge {
        HalfEdge {
            vertex: vertex,
            face: face,
            next: next,
            adjacent: None
        }
    }

    pub fn is_boundary(&self) -> bool {
        self.adjacent.is_some()
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
pub struct Face {
    // According to my reading you only need one.
    pub edge: Index,
}

impl Face {
    pub fn new(edge: Index) -> Face {
        Face { edge: edge }
    }
}

///////////////////////////////////////////////////////////////////////////////
// The central bucket of attributes and connectivity information
//
pub struct Mesh {
    // Attributes
    pub positions: Vec<Position>,
    pub colors: Option<Vec<Color>>,
    pub normals: Option<Vec<Normal>>,

    // Connectivity information
    pub faces: Vec<Face>,
    pub edges: Vec<HalfEdge>,
    pub vertices: Vec<Vertex>
}

///////////////////////////////////////////////////////////////////////////////
// NOTE: the following probably belong in the parent scope

pub fn icosahedron(radius: f32) -> Mesh {
    // const POINT_COUNT: u32 = 12;
    // const FACE_COUNT: u32 = 20;
    // const EDGE_COUNT: u32 = 60; // 30 * 2
    // const VERTEX_COUNT: u32 = 60; // 5 * 12

    // The coordinates of the iocosahedron are are described by the
    // cyclic permutations of (±ϕ, ±1, 0), where ϕ is the [Golden Ratio]
    // (https://en.wikipedia.org/wiki/Golden_ratio).
    let phi = (1.0 + f32::sqrt(5.0)) / 2.0;
    let positions = vec![
        math::set_radius(Point3::new( phi,  1.0,  0.0), radius),
        math::set_radius(Point3::new( phi, -1.0,  0.0), radius),
        math::set_radius(Point3::new(-phi,  1.0,  0.0), radius),
        math::set_radius(Point3::new(-phi, -1.0,  0.0), radius),
        math::set_radius(Point3::new( 0.0,  phi,  1.0), radius),
        math::set_radius(Point3::new( 0.0,  phi, -1.0), radius),
        math::set_radius(Point3::new( 0.0, -phi,  1.0), radius),
        math::set_radius(Point3::new( 0.0, -phi, -1.0), radius),
        math::set_radius(Point3::new( 1.0,  0.0,  phi), radius),
        math::set_radius(Point3::new(-1.0,  0.0,  phi), radius),
        math::set_radius(Point3::new( 1.0,  0.0, -phi), radius),
        math::set_radius(Point3::new(-1.0,  0.0, -phi), radius)
    ];
    let colors = None;
    let normals = None;

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
        // Face 0
        Vertex::new( 0, AttributeIndex::new( 0)),
        Vertex::new( 1, AttributeIndex::new( 1)),
        Vertex::new( 2, AttributeIndex::new( 2)),
        // Face 1
        Vertex::new( 3, AttributeIndex::new( 0)),
        Vertex::new( 4, AttributeIndex::new( 2)),
        Vertex::new( 5, AttributeIndex::new( 3)),
        // Face 2
        Vertex::new( 6, AttributeIndex::new( 0)),
        Vertex::new( 7, AttributeIndex::new( 3)),
        Vertex::new( 8, AttributeIndex::new( 4)),
        // Face 3
        Vertex::new( 9, AttributeIndex::new( 0)),
        Vertex::new(10, AttributeIndex::new( 4)),
        Vertex::new(11, AttributeIndex::new( 5)),
        // Face 4
        Vertex::new(12, AttributeIndex::new( 0)),
        Vertex::new(13, AttributeIndex::new( 5)),
        Vertex::new(14, AttributeIndex::new( 1)),
        // Face 5
        Vertex::new(15, AttributeIndex::new( 1)),
        Vertex::new(16, AttributeIndex::new( 6)),
        Vertex::new(17, AttributeIndex::new( 2)),
        // Face 6
        Vertex::new(18, AttributeIndex::new( 2)),
        Vertex::new(19, AttributeIndex::new( 7)),
        Vertex::new(20, AttributeIndex::new( 5)),
        // Face 7
        Vertex::new(21, AttributeIndex::new( 3)),
        Vertex::new(22, AttributeIndex::new( 8)),
        Vertex::new(23, AttributeIndex::new( 4)),
        // Face 8
        Vertex::new(24, AttributeIndex::new( 4)),
        Vertex::new(25, AttributeIndex::new( 9)),
        Vertex::new(26, AttributeIndex::new( 5)),
        // Face 9
        Vertex::new(27, AttributeIndex::new( 5)),
        Vertex::new(28, AttributeIndex::new(10)),
        Vertex::new(29, AttributeIndex::new( 1)),
        // Face 10
        Vertex::new(30, AttributeIndex::new( 2)),
        Vertex::new(31, AttributeIndex::new( 6)),
        Vertex::new(32, AttributeIndex::new( 7)),
        // Face 11
        Vertex::new(33, AttributeIndex::new( 3)),
        Vertex::new(34, AttributeIndex::new( 7)),
        Vertex::new(35, AttributeIndex::new( 8)),
        // Face 12
        Vertex::new(36, AttributeIndex::new( 4)),
        Vertex::new(37, AttributeIndex::new( 8)),
        Vertex::new(38, AttributeIndex::new( 9)),
        // Face 13
        Vertex::new(39, AttributeIndex::new( 5)),
        Vertex::new(40, AttributeIndex::new( 9)),
        Vertex::new(41, AttributeIndex::new(10)),
        // Face 14
        Vertex::new(42, AttributeIndex::new( 1)),
        Vertex::new(43, AttributeIndex::new(10)),
        Vertex::new(44, AttributeIndex::new( 6)),
        // Face 15
        Vertex::new(45, AttributeIndex::new( 6)),
        Vertex::new(46, AttributeIndex::new(11)),
        Vertex::new(47, AttributeIndex::new( 7)),
        // Face 16
        Vertex::new(48, AttributeIndex::new( 7)),
        Vertex::new(49, AttributeIndex::new(11)),
        Vertex::new(50, AttributeIndex::new( 8)),
        // Face 17
        Vertex::new(51, AttributeIndex::new( 8)),
        Vertex::new(52, AttributeIndex::new(11)),
        Vertex::new(53, AttributeIndex::new( 9)),
        // Face 18
        Vertex::new(54, AttributeIndex::new( 9)),
        Vertex::new(55, AttributeIndex::new(11)),
        Vertex::new(56, AttributeIndex::new(10)),
        // Face 19
        Vertex::new(57, AttributeIndex::new(10)),
        Vertex::new(58, AttributeIndex::new(11)),
        Vertex::new(59, AttributeIndex::new( 6)),
    ];

    let edges = vec![
        // Face 0     // vertex, face, next, adj
        HalfEdge::new(        0,    0,    1,  14),
        HalfEdge::new(        1,    0,    2,  17),
        HalfEdge::new(        2,    0,    0,   3),
        // Face 1
        HalfEdge::new(        3,    1,    4,   2),
        HalfEdge::new(        4,    1,    5,  23),
        HalfEdge::new(        5,    1,    3,   6),
        // Face 2
        HalfEdge::new(        6,    2,    7,   5),
        HalfEdge::new(        7,    2,    8,  29),
        HalfEdge::new(        8,    2,    6,   9),
        // Face 3
        HalfEdge::new(        9,    3,   10,   8),
        HalfEdge::new(       10,    3,   11,  35),
        HalfEdge::new(       11,    3,    9,  12),
        // Face 4
        HalfEdge::new(       12,    4,   13,  11),
        HalfEdge::new(       13,    4,   14,  41),
        HalfEdge::new(       14,    4,   12,   0),
        // Face 5
        HalfEdge::new(       15,    5,   16,  44),
        HalfEdge::new(       16,    5,   17,  18),
        HalfEdge::new(       17,    5,   15,   1),
        // Face 6
        HalfEdge::new(       18,    6,   19,  16),
        HalfEdge::new(       19,    6,   20,  47),
        HalfEdge::new(       20,    6,   18,  21),
        // Face 7
        HalfEdge::new(       21,    7,   22,  20),
        HalfEdge::new(       22,    7,   23,  24),
        HalfEdge::new(       23,    7,   21,   4),
        // Face 8
        HalfEdge::new(       24,    8,   25,  22),
        HalfEdge::new(       25,    8,   26,  50),
        HalfEdge::new(       26,    8,   24,  27),
        // Face 9
        HalfEdge::new(       27,    9,   28,  26),
        HalfEdge::new(       28,    9,   29,  30),
        HalfEdge::new(       29,    9,   27,   7),
        // Face 10
        HalfEdge::new(       30,   10,   31,  29),
        HalfEdge::new(       31,   10,   32,  53),
        HalfEdge::new(       32,   10,   30,  33),
        // Face 11
        HalfEdge::new(       33,   11,   34,  32),
        HalfEdge::new(       34,   11,   35,  36),
        HalfEdge::new(       35,   11,   33,  10),
        // Face 12
        HalfEdge::new(       36,   12,   37,  34),
        HalfEdge::new(       37,   12,   38,  56),
        HalfEdge::new(       38,   12,   36,  39),
        // Face 13
        HalfEdge::new(       39,   13,   40,  38),
        HalfEdge::new(       40,   13,   41,  42),
        HalfEdge::new(       41,   13,   39,  13),
        // Face 14
        HalfEdge::new(       42,   14,   43,  40),
        HalfEdge::new(       43,   14,   44,  59),
        HalfEdge::new(       44,   14,   42,  15),
        // Face 15
        HalfEdge::new(       45,   15,   46,  58),
        HalfEdge::new(       46,   15,   47,  48),
        HalfEdge::new(       47,   15,   45,  19),
        // Face 16
        HalfEdge::new(       48,   16,   49,  46),
        HalfEdge::new(       49,   16,   50,  51),
        HalfEdge::new(       50,   16,   48,  25),
        // Face 17
        HalfEdge::new(       51,   17,   52,  49),
        HalfEdge::new(       52,   17,   53,  54),
        HalfEdge::new(       53,   17,   51,  31),
        // Face 18
        HalfEdge::new(       54,   18,   55,  52),
        HalfEdge::new(       55,   18,   56,  57),
        HalfEdge::new(       56,   18,   54,  37),
        // Face 19
        HalfEdge::new(       57,   19,   58,  55),
        HalfEdge::new(       58,   19,   59,  45),
        HalfEdge::new(       59,   19,   57,  43),
    ];

    Mesh {
        positions: positions,
        colors: colors,
        normals: normals,

        vertices: vertices,
        faces: faces,
        edges: edges
    }
}
