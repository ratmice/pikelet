use cgmath::Point3;
use math;

use super::*;

pub fn triangle(scale: f32) -> Mesh {
    let extent = scale / 2.0;
    let positions = vec![Point3::new(0.0, extent, 0.0),
                         Point3::new(-extent, -extent, 0.0),
                         Point3::new(extent, -extent, 0.0)];

    let faces = vec![Face::new(0)];

    let edges =
        vec![Edge::new_boundary(0, 0, 1), Edge::new_boundary(1, 0, 2), Edge::new_boundary(2, 0, 0)];

    Mesh {
        positions: positions,
        edges: edges,
        faces: faces,
    }
}

pub fn plane(scale: f32) -> Mesh {
    let extent = scale / 2.0;
    let positions = vec![Point3::new(-extent, extent, 0.0),
                         Point3::new(-extent, -extent, 0.0),
                         Point3::new(extent, -extent, 0.0),
                         Point3::new(extent, extent, 0.0)];

    let faces = vec![Face::new(0), Face::new(3)];

    let edges = vec![Edge::new_boundary(0, 0, 1),
                     Edge::new_boundary(1, 0, 2),
                     Edge::new(2, 0, 0, 3),
                     Edge::new(0, 1, 4, 2),
                     Edge::new_boundary(2, 1, 5),
                     Edge::new_boundary(3, 1, 3)];

    Mesh {
        positions: positions,
        edges: edges,
        faces: faces,
    }
}

pub fn tetrahedron(scale: f32) -> Mesh {
    let extent = scale / 2.0;

    let positions = vec![Point3::new(extent, extent, extent),
                         Point3::new(extent, -extent, -extent),
                         Point3::new(-extent, extent, -extent),
                         Point3::new(-extent, -extent, extent)];

    let faces = vec![Face::new(0), Face::new(3), Face::new(6), Face::new(9)];

    let edges = vec![
        // Face 0     // point, face, next, adj
        Edge::new(       0,    0,    1,  8), //  0
        Edge::new(       3,    0,    2, 10), //  1
        Edge::new(       1,    0,    0,  3), //  2
        // Face 1
        Edge::new(       0,    1,    4,  2), //  3
        Edge::new(       1,    1,    5,  9), //  4
        Edge::new(       2,    1,    3,  6), //  5
        // Face 2
        Edge::new(       0,    2,    7,  5), //  6
        Edge::new(       2,    2,    8, 11), //  7
        Edge::new(       3,    2,    6,  0), //  8
        // Face 3
        Edge::new(       2,    3,   10,  4), //  9
        Edge::new(       1,    3,   11,  1), // 10
        Edge::new(       3,    3,    9,  7), // 11
    ];

    Mesh {
        positions: positions,
        edges: edges,
        faces: faces,
    }
}

pub fn icosahedron(radius: f32) -> Mesh {
    // The coordinates of the iocosahedron are are described by the
    // cyclic permutations of (±ϕ, ±1, 0), where ϕ is the [Golden Ratio]
    // (https://en.wikipedia.org/wiki/Golden_ratio).
    // NOTE: The order here is such that 0 - 11 line up with the logical
    //       winding of faces over the "net" of the polyhedron.
    //       (would be cool if you could annotate source code with images
    //        to explain notes like this!)
    let phi = (1.0 + f32::sqrt(5.0)) / 2.0;
    let positions = vec![math::set_radius(Point3::new(0.0, phi, -1.0), radius),
                         math::set_radius(Point3::new(-phi, 1.0, 0.0), radius),
                         math::set_radius(Point3::new(0.0, phi, 1.0), radius),
                         math::set_radius(Point3::new(phi, 1.0, 0.0), radius),
                         math::set_radius(Point3::new(1.0, 0.0, -phi), radius),
                         math::set_radius(Point3::new(-1.0, 0.0, -phi), radius),
                         math::set_radius(Point3::new(-1.0, 0.0, phi), radius),
                         math::set_radius(Point3::new(1.0, 0.0, phi), radius),
                         math::set_radius(Point3::new(phi, -1.0, 0.0), radius),
                         math::set_radius(Point3::new(0.0, -phi, -1.0), radius),
                         math::set_radius(Point3::new(-phi, -1.0, 0.0), radius),
                         math::set_radius(Point3::new(0.0, -phi, 1.0), radius)];

    // Edges around the face:
    let faces = vec![
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

    let edges = vec![
        // Face 0     // point, face, next, adj
        Edge::new(       0,    0,    1,  14), // 0
        Edge::new(       1,    0,    2,  17), // 1
        Edge::new(       2,    0,    0,   3), // 2
        // Face 1
        Edge::new(       0,    1,    4,   2), // 3
        Edge::new(       2,    1,    5,  20), // 4
        Edge::new(       3,    1,    3,   6), // 5
        // Face 2
        Edge::new(       0,    2,    7,   5), // 6
        Edge::new(       3,    2,    8,  23), // 7
        Edge::new(       4,    2,    6,   9), // 8
        // Face 3
        Edge::new(       0,    3,   10,   8), // 9
        Edge::new(       4,    3,   11,  26), // 10
        Edge::new(       5,    3,    9,  12), // 11
        // Face 4
        Edge::new(       0,    4,   13,  11), // 12
        Edge::new(       5,    4,   14,  29), // 13
        Edge::new(       1,    4,   12,   0), // 14
        // Face 5
        Edge::new(       1,    5,   16,  44), // 15
        Edge::new(       6,    5,   17,  30), // 16
        Edge::new(       2,    5,   15,   1), // 17
        // Face 6
        Edge::new(       2,    6,   19,  32), // 18
        Edge::new(       7,    6,   20,  33), // 19
        Edge::new(       3,    6,   18,   4), // 20
        // Face 7
        Edge::new(       3,    7,   22,  35), // 21
        Edge::new(       8,    7,   23,  36), // 22
        Edge::new(       4,    7,   21,   7), // 23
        // Face 8
        Edge::new(       4,    8,   25,  38), // 24
        Edge::new(       9,    8,   26,  39), // 25
        Edge::new(       5,    8,   24,  10), // 26
        // Face 9
        Edge::new(       5,    9,   28,  41), // 27
        Edge::new(      10,    9,   29,  42), // 28
        Edge::new(       1,    9,   27,  13), // 29
        // Face 10
        Edge::new(       2,   10,   31,  16), // 30
        Edge::new(       6,   10,   32,  47), // 31
        Edge::new(       7,   10,   30,  18), // 32
        // Face 11
        Edge::new(       3,   11,   34,  19), // 33
        Edge::new(       7,   11,   35,  50), // 34
        Edge::new(       8,   11,   33,  21), // 35
        // Face 12
        Edge::new(       4,   12,   37,  22), // 36
        Edge::new(       8,   12,   38,  53), // 37
        Edge::new(       9,   12,   36,  24), // 38
        // Face 13
        Edge::new(       5,   13,   40,  25), // 39
        Edge::new(       9,   13,   41,  56), // 40
        Edge::new(      10,   13,   39,  27), // 41
        // Face 14
        Edge::new(       1,   14,   43,  28), // 42
        Edge::new(      10,   14,   44,  59), // 43
        Edge::new(       6,   14,   42,  15), // 44
        // Face 15
        Edge::new(       6,   15,   46,  58), // 45
        Edge::new(      11,   15,   47,  48), // 46
        Edge::new(       7,   15,   45,  31), // 47
        // Face 16
        Edge::new(       7,   16,   49,  46), // 48
        Edge::new(      11,   16,   50,  51), // 49
        Edge::new(       8,   16,   48,  34), // 50
        // Face 17
        Edge::new(       8,   17,   52,  49), // 51
        Edge::new(      11,   17,   53,  54), // 52
        Edge::new(       9,   17,   51,  37), // 53
        // Face 18
        Edge::new(       9,   18,   55,  52), // 54
        Edge::new(      11,   18,   56,  57), // 55
        Edge::new(      10,   18,   54,  40), // 56
        // Face 19
        Edge::new(      10,   19,   58,  55), // 57
        Edge::new(      11,   19,   59,  45), // 58
        Edge::new(       6,   19,   57,  43), // 59
    ];

    Mesh {
        positions: positions,
        faces: faces,
        edges: edges,
    }
}
