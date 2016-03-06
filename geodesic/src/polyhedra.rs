//! Some regular polyhedra

use cgmath::Point3;

use {math, get_mut};
use {Edge, Face, Geometry, Node};
use EdgeIndex as E;
use FaceIndex as F;
use NodeIndex as N;

/// The base geometry of a [regular iocosahedron](https://en.wikipedia.org/wiki/Regular_icosahedron).
pub fn icosahedron() -> Geometry {
    // The coordinates of the iocosahedron are are described by the
    // cyclic permutations of (±ϕ, ±1, 0), where ϕ is the [Golden Ratio]
    // (https://en.wikipedia.org/wiki/Golden_ratio).
    let phi = (1.0 + f32::sqrt(5.0)) / 2.0;
    let mut nodes = vec![
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

    let mut edges = vec![
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

    for (index, edge) in edges.iter().enumerate() {
        get_mut(&mut nodes, edge.nodes[0]).edges.push(E(index));
        get_mut(&mut nodes, edge.nodes[1]).edges.push(E(index));
    }

    for (index, face) in faces.iter().enumerate() {
        get_mut(&mut nodes, face.nodes[0]).faces.push(F(index));
        get_mut(&mut nodes, face.nodes[1]).faces.push(F(index));
        get_mut(&mut nodes, face.nodes[2]).faces.push(F(index));
    }

    for (index, face) in faces.iter().enumerate() {
        get_mut(&mut edges, face.edges[0]).faces.push(F(index));
        get_mut(&mut edges, face.edges[1]).faces.push(F(index));
        get_mut(&mut edges, face.edges[2]).faces.push(F(index));
    }

    Geometry {
        nodes: nodes,
        edges: edges,
        faces: faces,
    }
}

// /// The base geometry of a [regular octahedron](https://en.wikipedia.org/wiki/Octahedron).
// pub fn octahedron() -> Geometry {
//     let nodes = vec![
//         // North pole
//         Node { position: math::set_radius(Point3::new( 0.0,  0.0,  1.0), 1.0) },
//         // Equator
//         Node { position: math::set_radius(Point3::new( 1.0,  0.0,  0.0), 1.0) },
//         Node { position: math::set_radius(Point3::new( 0.0,  1.0,  0.0), 1.0) },
//         Node { position: math::set_radius(Point3::new(-1.0,  0.0,  0.0), 1.0) },
//         Node { position: math::set_radius(Point3::new( 0.0, -1.0,  0.0), 1.0) },
//         // South pole
//         Node { position: math::set_radius(Point3::new( 0.0,  0.0, -1.0), 1.0) },
//     ];

//     let edges = vec![
//         // South
//         [N(5), N(2)], [N(5), N(3)], [N(5), N(4)], [N(5), N(1)],
//         // Equator
//         [N(1), N(2)], [N(2), N(3)], [N(3), N(4)], [N(4), N(1)],
//         // North
//         [N(0), N(4)], [N(0), N(3)], [N(0), N(2)], [N(0), N(1)],
//     ];

//     let faces = vec![
//         // Southern hemisphere
//         [N(1), N(5), N(2)],
//         [N(2), N(5), N(3)],
//         [N(3), N(5), N(4)],
//         [N(4), N(5), N(1)],
//         // Northern Hemisphere
//         [N(1), N(0), N(4)],
//         [N(4), N(0), N(3)],
//         [N(3), N(0), N(2)],
//         [N(2), N(0), N(1)],
//     ];

//     Geometry {
//         nodes: nodes,
//         edges: edges,
//         faces: faces,
//     }
// }
