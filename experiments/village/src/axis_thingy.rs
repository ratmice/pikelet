// Copyright Brendan Zabarauskas 2014

use shader::Vertex;

const X_AXIS_COLOR: [f32, ..3] = [0.9, 0.1, 0.1];
const Y_AXIS_COLOR: [f32, ..3] = [0.1, 0.9, 0.1];
const Z_AXIS_COLOR: [f32, ..3] = [0.1, 0.1, 0.9];

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
    Vertex { pos: [-1.0,  0.0,  0.0], color: X_AXIS_COLOR },
    Vertex { pos: [ 1.0,  0.0,  0.0], color: X_AXIS_COLOR },
    // Y axis
    Vertex { pos: [ 0.0, -1.0,  0.0], color: Y_AXIS_COLOR },
    Vertex { pos: [ 0.0,  1.0,  0.0], color: Y_AXIS_COLOR },
    // Z axis
    Vertex { pos: [ 0.0,  0.0, -1.0], color: Z_AXIS_COLOR },
    Vertex { pos: [ 0.0,  0.0,  1.0], color: Z_AXIS_COLOR },
];
