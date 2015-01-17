// Copyright The Voyager Developers 2014

pub mod axis {
    use shader::color::Vertex;

    pub const X_AXIS_COLOR: [f32; 3] = [0.9, 0.1, 0.1];
    pub const Y_AXIS_COLOR: [f32; 3] = [0.1, 0.9, 0.1];
    pub const Z_AXIS_COLOR: [f32; 3] = [0.1, 0.1, 0.9];

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
}

pub mod water {
    use shader::color::Vertex;
    use sky;

    pub const COLOR: [f32; 3] = [sky::DAY_COLOR[0],
                                   sky::DAY_COLOR[1],
                                   sky::DAY_COLOR[2]];

    pub const VERTEX_DATA: &'static [Vertex] = &[
        Vertex { pos: [-1.0, -1.0,  0.0], color: COLOR },
        Vertex { pos: [ 1.0, -1.0,  0.0], color: COLOR },
        Vertex { pos: [ 1.0,  1.0,  0.0], color: COLOR },
        Vertex { pos: [-1.0,  1.0,  0.0], color: COLOR },
    ];

    pub const INDEX_DATA: &'static [u8] = &[
         0,  1,  2,  2,  3,  0,
    ];
}


pub mod house {
    use shader::flat::Vertex;

    pub const COLOR: [f32; 3] = [1.0, 1.0, 1.0];

    pub const VERTEX_DATA: &'static [Vertex] = &[
        // top (0, 0, 1)
        Vertex { pos: [-1.0, -1.0,  1.0], norm: [ 0.0,  0.0,  1.0], color: COLOR },
        Vertex { pos: [ 1.0, -1.0,  1.0], norm: [ 0.0,  0.0,  1.0], color: COLOR },
        Vertex { pos: [ 1.0,  1.0,  1.0], norm: [ 0.0,  0.0,  1.0], color: COLOR },
        Vertex { pos: [-1.0,  1.0,  1.0], norm: [ 0.0,  0.0,  1.0], color: COLOR },
        // bottom (0, 0, -1)
        Vertex { pos: [-1.0,  1.0, -1.0], norm: [ 0.0,  0.0, -1.0], color: COLOR },
        Vertex { pos: [ 1.0,  1.0, -1.0], norm: [ 0.0,  0.0, -1.0], color: COLOR },
        Vertex { pos: [ 1.0, -1.0, -1.0], norm: [ 0.0,  0.0, -1.0], color: COLOR },
        Vertex { pos: [-1.0, -1.0, -1.0], norm: [ 0.0,  0.0, -1.0], color: COLOR },
        // right (1, 0, 0)
        Vertex { pos: [ 1.0, -1.0, -1.0], norm: [ 1.0,  0.0,  0.0], color: COLOR },
        Vertex { pos: [ 1.0,  1.0, -1.0], norm: [ 1.0,  0.0,  0.0], color: COLOR },
        Vertex { pos: [ 1.0,  1.0,  1.0], norm: [ 1.0,  0.0,  0.0], color: COLOR },
        Vertex { pos: [ 1.0, -1.0,  1.0], norm: [ 1.0,  0.0,  0.0], color: COLOR },
        // left (-1, 0, 0)
        Vertex { pos: [-1.0, -1.0,  1.0], norm: [-1.0,  0.0,  0.0], color: COLOR },
        Vertex { pos: [-1.0,  1.0,  1.0], norm: [-1.0,  0.0,  0.0], color: COLOR },
        Vertex { pos: [-1.0,  1.0, -1.0], norm: [-1.0,  0.0,  0.0], color: COLOR },
        Vertex { pos: [-1.0, -1.0, -1.0], norm: [-1.0,  0.0,  0.0], color: COLOR },
        // front (0, 1, 0)
        Vertex { pos: [ 1.0,  1.0, -1.0], norm: [ 0.0,  1.0,  0.0], color: COLOR },
        Vertex { pos: [-1.0,  1.0, -1.0], norm: [ 0.0,  1.0,  0.0], color: COLOR },
        Vertex { pos: [-1.0,  1.0,  1.0], norm: [ 0.0,  1.0,  0.0], color: COLOR },
        Vertex { pos: [ 1.0,  1.0,  1.0], norm: [ 0.0,  1.0,  0.0], color: COLOR },
        // back (0, -1, 0)
        Vertex { pos: [ 1.0, -1.0,  1.0], norm: [ 0.0, -1.0,  0.0], color: COLOR },
        Vertex { pos: [-1.0, -1.0,  1.0], norm: [ 0.0, -1.0,  0.0], color: COLOR },
        Vertex { pos: [-1.0, -1.0, -1.0], norm: [ 0.0, -1.0,  0.0], color: COLOR },
        Vertex { pos: [ 1.0, -1.0, -1.0], norm: [ 0.0, -1.0,  0.0], color: COLOR },
    ];

    pub const INDEX_DATA: &'static [u8] = &[
         0,  1,  2,  2,  3,  0, // top
         4,  5,  6,  6,  7,  4, // bottom
         8,  9, 10, 10, 11,  8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ];
}

pub mod antenna {
    use shader::color::Vertex;

    pub const COLOR: [f32; 3] = [0.2, 0.2, 0.2];

    /// Points for an antenna
    pub const VERTEX_DATA: &'static [Vertex] = &[
        // Cross X
        Vertex { pos: [-1.0,  0.0,  0.9], color: COLOR },
        Vertex { pos: [ 1.0,  0.0,  0.9], color: COLOR },
        // Cross Y
        Vertex { pos: [ 0.0, -1.0,  0.9], color: COLOR },
        Vertex { pos: [ 0.0,  1.0,  0.9], color: COLOR },
        // X axis
        Vertex { pos: [ 0.0,  0.0, -1.0], color: COLOR },
        Vertex { pos: [ 0.0,  0.0,  1.0], color: COLOR },
    ];
}

pub mod tree {
    pub mod trunk {
        use shader::color::Vertex;

        pub const COLOR: [f32; 3] = [0.2, 0.2, 0.2];

        pub const VERTEX_DATA: &'static [Vertex] = &[
            Vertex { pos: [ 0.0,  0.0,  0.01], color: COLOR },
            Vertex { pos: [ 0.0,  0.7,  0.01], color: COLOR },
        ];
    }

    pub mod foliage {
        use shader::color::Vertex;

        pub const COLOR: [f32; 3] = [0.2, 0.4, 0.1];

        pub const VERTEX_DATA: &'static [Vertex] = &[
            Vertex { pos: [-0.2,  0.3,  0.0], color: COLOR },
            Vertex { pos: [ 0.2,  0.3,  0.0], color: COLOR },
            Vertex { pos: [ 0.0,  1.0,  0.0], color: COLOR },
        ];
    }
}
