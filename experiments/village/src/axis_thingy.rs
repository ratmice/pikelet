// Copyright Brendan Zabarauskas 2014

use Vertex;

/// Points for an axis widget thingy
///
/// ~~~nottrust
///          [0, 1, 0]
///              |[0, 0, 1]
///              |  /
///              | /
/// [-1, 0, 0]   |/    [1, 0, 0]
///      --------o----------
///             /|
///            / |
///           /  |
///          /   |
///   [0, 0, -1] |
///          [0, -1, 0]
/// ~~~~
pub const VERTEX_DATA: &'static [Vertex] = &[
    // X axis
    Vertex { pos: [-1.0,  0.0,  0.0], color: [1.0, 0.5, 0.5] },
    Vertex { pos: [ 1.0,  0.0,  0.0], color: [1.0, 0.5, 0.5] },
    // Y axis
    Vertex { pos: [ 0.0, -1.0,  0.0], color: [0.5, 1.0, 0.5] },
    Vertex { pos: [ 0.0,  1.0,  0.0], color: [0.5, 1.0, 0.5] },
    // Z axis
    Vertex { pos: [ 0.0,  0.0, -1.0], color: [0.5, 0.5, 1.0] },
    Vertex { pos: [ 0.0,  0.0,  1.0], color: [0.5, 0.5, 1.0] },
];
