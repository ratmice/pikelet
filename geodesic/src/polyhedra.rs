//! Some regular polyhedra

use cgmath::Point3;

use Geometry;

/// The base geometry of a [regular iocosahedron]
/// (https://en.wikipedia.org/wiki/Regular_icosahedron) with an edge length of 2.
pub fn icosahedron() -> Geometry {
    // The coordinates of the iocosahedron are are described by the
    // cyclic permutations of (±ϕ, ±1, 0), where ϕ is the [Golden Ratio]
    // (https://en.wikipedia.org/wiki/Golden_ratio).
    let phi = (1.0 + f32::sqrt(5.0)) / 2.0;
    let points = vec![
        Point3::new( phi,  1.0,  0.0),
        Point3::new( phi, -1.0,  0.0),
        Point3::new(-phi,  1.0,  0.0),
        Point3::new(-phi, -1.0,  0.0),
        Point3::new( 0.0,  phi,  1.0),
        Point3::new( 0.0,  phi, -1.0),
        Point3::new( 0.0, -phi,  1.0),
        Point3::new( 0.0, -phi, -1.0),
        Point3::new( 1.0,  0.0,  phi),
        Point3::new(-1.0,  0.0,  phi),
        Point3::new( 1.0,  0.0, -phi),
        Point3::new(-1.0,  0.0, -phi),
    ];

    let edges = vec![
        [ 0,  1], [ 0,  4], [ 0,  5], [ 0,  8], [ 0, 10],
        [ 1,  6], [ 1,  7], [ 1,  8], [ 1, 10], [ 2,  3],
        [ 2,  4], [ 2,  5], [ 2,  9], [ 2, 11], [ 3,  6],
        [ 3,  7], [ 3,  9], [ 3, 11], [ 4,  5], [ 4,  8],
        [ 4,  9], [ 5, 10], [ 5, 11], [ 6,  7], [ 6,  8],
        [ 6,  9], [ 7, 10], [ 7, 11], [ 8,  9], [10, 11],
    ];

    let faces = vec![
        [ 8,  1,  0],
        [ 5,  4,  0],
        [10,  5,  0],
        [ 4,  8,  0],
        [ 1, 10,  0],
        [ 8,  6,  1],
        [ 6,  7,  1],
        [ 7, 10,  1],
        [11,  3,  2],
        [ 9,  4,  2],
        [ 4,  5,  2],
        [ 3,  9,  2],
        [ 5, 11,  2],
        [ 7,  6,  3],
        [11,  7,  3],
        [ 6,  9,  3],
        [ 9,  8,  4],
        [10, 11,  5],
        [ 8,  9,  6],
        [11, 10,  7],
    ];

    Geometry {
        points: points,
        edges: edges,
        faces: faces,
    }
}

/// The base geometry of a [regular octahedron]
/// (https://en.wikipedia.org/wiki/Octahedron) with an edge length of sqrt(2).
pub fn octahedron() -> Geometry {
    let points = vec![
        // North pole
        Point3::new( 0.0,  0.0,  1.0),
        // Equator
        Point3::new( 1.0,  0.0,  0.0),
        Point3::new( 0.0,  1.0,  0.0),
        Point3::new(-1.0,  0.0,  0.0),
        Point3::new( 0.0, -1.0,  0.0),
        // South pole
        Point3::new( 0.0,  0.0, -1.0),
    ];

    let edges = vec![
        // South
        [5, 2], [5, 3], [5, 4], [5, 1],
        // Equator
        [1, 2], [2, 3], [3, 4], [4, 1],
        // North
        [0, 4], [0, 3], [0, 2], [0, 1],
    ];

    let faces = vec![
        // Southern hemisphere
        [1, 5, 2],
        [2, 5, 3],
        [3, 5, 4],
        [4, 5, 1],
        // Northern Hemisphere
        [1, 0, 4],
        [4, 0, 3],
        [3, 0, 2],
        [2, 0, 1],
    ];

    Geometry {
        points: points,
        edges: edges,
        faces: faces,
    }
}
