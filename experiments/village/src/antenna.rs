// Copyright The Voyager Developers 2014

use shader::color::Vertex;

const ANTENNA_COLOR: [f32, ..3] = [0.2, 0.2, 0.2];

/// Points for an antenna
pub const VERTEX_DATA: &'static [Vertex] = &[
    // Cross X
    Vertex { pos: [-1.0,  0.0,  0.9], color: ANTENNA_COLOR },
    Vertex { pos: [ 1.0,  0.0,  0.9], color: ANTENNA_COLOR },
    // Cross Y
    Vertex { pos: [ 0.0, -1.0,  0.9], color: ANTENNA_COLOR },
    Vertex { pos: [ 0.0,  1.0,  0.9], color: ANTENNA_COLOR },
    // X axis
    Vertex { pos: [ 0.0,  0.0, -1.0], color: ANTENNA_COLOR },
    Vertex { pos: [ 0.0,  0.0,  1.0], color: ANTENNA_COLOR },
];


