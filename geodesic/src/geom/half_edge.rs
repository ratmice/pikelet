
use cgmath::{Vector4, Point3};

///////////////////////////////////////////////////////////////////////////////
// Some basic type aliases in order to attemp self-documentation

pub type Index = u32;
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
    position: Index,        // Positions are required.
    color: Option<Index>,   // If a mesh has vertex colors.
    normal: Option<Index>,  // If a mesh has normals.
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
    attributes: AttributeIndex,

    // The HalfEdge eminating from this vertex.
    edge: Index,
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
    vertex: Index,

    // The face that this edge borders.
    face: Index,

    // The next HalfEdge around the face.
    next: Index,

    // Oppositely oriented adjacent HalfEdge.
    // If this is None then we have a boundary edge.
    adjacent_edge: Option<Index>,
}

///////////////////////////////////////////////////////////////////////////////
// The Face
//
// TODO: Is the face really so sparse?
//
pub struct Face {
    // According to my reading you only need one.
    edge: Index,
}

///////////////////////////////////////////////////////////////////////////////
// The central bucket of attributes and connectivity information
//
pub struct Mesh {
    // Attributes
    positions: Vec<Position>,
    color: Option<Vec<Color>>,
    normal: Option<Vec<Normal>>,

    // Connectivity information
    faces: Vec<Face>,
    edges: Vec<HalfEdge>,
    vertices: Vec<Vertex>
}
