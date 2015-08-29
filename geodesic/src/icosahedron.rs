//! The base geometry of a regular icosahedron

use na::Pnt3;

/// Generates the cartesian coordinates of a [regular iocosahedron]
/// (https://en.wikipedia.org/wiki/Regular_icosahedron) with an edge length of 2.
pub fn points() -> [Pnt3<f32>; 12] {
    // The cartesian coordinates of the iocosahedron are are described by the
    // cyclic permutations of (±ϕ, ±1, 0), where ϕ is the [Golden Ratio]
    // (https://en.wikipedia.org/wiki/Golden_ratio).

    let phi = (1.0 + f32::sqrt(5.0)) / 2.0;
    [
        Pnt3::new( phi,  1.0,  0.0),
        Pnt3::new( phi, -1.0,  0.0),
        Pnt3::new(-phi,  1.0,  0.0),
        Pnt3::new(-phi, -1.0,  0.0),
        Pnt3::new( 0.0,  phi,  1.0),
        Pnt3::new( 0.0,  phi, -1.0),
        Pnt3::new( 0.0, -phi,  1.0),
        Pnt3::new( 0.0, -phi, -1.0),
        Pnt3::new( 1.0,  0.0,  phi),
        Pnt3::new(-1.0,  0.0,  phi),
        Pnt3::new( 1.0,  0.0, -phi),
        Pnt3::new(-1.0,  0.0, -phi),
    ]
}

pub fn edges() -> [[u8; 2]; 30] {
    [
        [ 0,  1], [ 0,  4], [ 0,  5], [ 0,  8], [ 0, 10],
        [ 1,  6], [ 1,  7], [ 1,  8], [ 1, 10], [ 2,  3],
        [ 2,  4], [ 2,  5], [ 2,  9], [ 2, 11], [ 3,  6],
        [ 3,  7], [ 3,  9], [ 3, 11], [ 4,  5], [ 4,  8],
        [ 4,  9], [ 5, 10], [ 5, 11], [ 6,  7], [ 6,  8],
        [ 6,  9], [ 7, 10], [ 7, 11], [ 8,  9], [10, 11],
    ]
}

pub fn faces() -> [[u8; 3]; 20] {
    [
        [ 0,  1,  8],
        [ 0,  4,  5],
        [ 0,  5, 10],
        [ 0,  8,  4],
        [ 0, 10,  1],
        [ 1,  6,  8],
        [ 1,  7,  6],
        [ 1, 10,  7],
        [ 2,  3, 11],
        [ 2,  4,  9],
        [ 2,  5,  4],
        [ 2,  9,  3],
        [ 2, 11,  5],
        [ 3,  6,  7],
        [ 3,  7, 11],
        [ 3,  9,  6],
        [ 4,  8,  9],
        [ 5, 11, 10],
        [ 6,  9,  8],
        [ 7, 10, 11],
    ]
}
