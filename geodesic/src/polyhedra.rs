//! Some regular polyhedra

use cgmath::Point3;

use math;
use {Geometry, Node};
use NodeIndex as N;

/// The base geometry of a [regular iocosahedron](https://en.wikipedia.org/wiki/Regular_icosahedron).
pub fn icosahedron() -> Geometry {
    // The coordinates of the iocosahedron are are described by the
    // cyclic permutations of (±ϕ, ±1, 0), where ϕ is the [Golden Ratio]
    // (https://en.wikipedia.org/wiki/Golden_ratio).
    let phi = (1.0 + f32::sqrt(5.0)) / 2.0;
    let nodes = vec![
        Node { position: math::set_radius(Point3::new( phi,  1.0,  0.0), 1.0) },
        Node { position: math::set_radius(Point3::new( phi, -1.0,  0.0), 1.0) },
        Node { position: math::set_radius(Point3::new(-phi,  1.0,  0.0), 1.0) },
        Node { position: math::set_radius(Point3::new(-phi, -1.0,  0.0), 1.0) },
        Node { position: math::set_radius(Point3::new( 0.0,  phi,  1.0), 1.0) },
        Node { position: math::set_radius(Point3::new( 0.0,  phi, -1.0), 1.0) },
        Node { position: math::set_radius(Point3::new( 0.0, -phi,  1.0), 1.0) },
        Node { position: math::set_radius(Point3::new( 0.0, -phi, -1.0), 1.0) },
        Node { position: math::set_radius(Point3::new( 1.0,  0.0,  phi), 1.0) },
        Node { position: math::set_radius(Point3::new(-1.0,  0.0,  phi), 1.0) },
        Node { position: math::set_radius(Point3::new( 1.0,  0.0, -phi), 1.0) },
        Node { position: math::set_radius(Point3::new(-1.0,  0.0, -phi), 1.0) },
    ];

    let edges = vec![
        [N( 0), N( 1)], [N( 0), N( 4)], [N( 0), N( 5)], [N( 0), N( 8)], [N( 0), N(10)],
        [N( 1), N( 6)], [N( 1), N( 7)], [N( 1), N( 8)], [N( 1), N(10)], [N( 2), N( 3)],
        [N( 2), N( 4)], [N( 2), N( 5)], [N( 2), N( 9)], [N( 2), N(11)], [N( 3), N( 6)],
        [N( 3), N( 7)], [N( 3), N( 9)], [N( 3), N(11)], [N( 4), N( 5)], [N( 4), N( 8)],
        [N( 4), N( 9)], [N( 5), N(10)], [N( 5), N(11)], [N( 6), N( 7)], [N( 6), N( 8)],
        [N( 6), N( 9)], [N( 7), N(10)], [N( 7), N(11)], [N( 8), N( 9)], [N(10), N(11)],
    ];

    let faces = vec![
        [N( 8), N( 1), N( 0)],
        [N( 5), N( 4), N( 0)],
        [N(10), N( 5), N( 0)],
        [N( 4), N( 8), N( 0)],
        [N( 1), N(10), N( 0)],
        [N( 8), N( 6), N( 1)],
        [N( 6), N( 7), N( 1)],
        [N( 7), N(10), N( 1)],
        [N(11), N( 3), N( 2)],
        [N( 9), N( 4), N( 2)],
        [N( 4), N( 5), N( 2)],
        [N( 3), N( 9), N( 2)],
        [N( 5), N(11), N( 2)],
        [N( 7), N( 6), N( 3)],
        [N(11), N( 7), N( 3)],
        [N( 6), N( 9), N( 3)],
        [N( 9), N( 8), N( 4)],
        [N(10), N(11), N( 5)],
        [N( 8), N( 9), N( 6)],
        [N(11), N(10), N( 7)],
    ];

    Geometry {
        nodes: nodes,
        edges: edges,
        faces: faces,
    }
}

/// The base geometry of a [regular octahedron](https://en.wikipedia.org/wiki/Octahedron).
pub fn octahedron() -> Geometry {
    let nodes = vec![
        // North pole
        Node { position: math::set_radius(Point3::new( 0.0,  0.0,  1.0), 1.0) },
        // Equator
        Node { position: math::set_radius(Point3::new( 1.0,  0.0,  0.0), 1.0) },
        Node { position: math::set_radius(Point3::new( 0.0,  1.0,  0.0), 1.0) },
        Node { position: math::set_radius(Point3::new(-1.0,  0.0,  0.0), 1.0) },
        Node { position: math::set_radius(Point3::new( 0.0, -1.0,  0.0), 1.0) },
        // South pole
        Node { position: math::set_radius(Point3::new( 0.0,  0.0, -1.0), 1.0) },
    ];

    let edges = vec![
        // South
        [N(5), N(2)], [N(5), N(3)], [N(5), N(4)], [N(5), N(1)],
        // Equator
        [N(1), N(2)], [N(2), N(3)], [N(3), N(4)], [N(4), N(1)],
        // North
        [N(0), N(4)], [N(0), N(3)], [N(0), N(2)], [N(0), N(1)],
    ];

    let faces = vec![
        // Southern hemisphere
        [N(1), N(5), N(2)],
        [N(2), N(5), N(3)],
        [N(3), N(5), N(4)],
        [N(4), N(5), N(1)],
        // Northern Hemisphere
        [N(1), N(0), N(4)],
        [N(4), N(0), N(3)],
        [N(3), N(0), N(2)],
        [N(2), N(0), N(1)],
    ];

    Geometry {
        nodes: nodes,
        edges: edges,
        faces: faces,
    }
}
