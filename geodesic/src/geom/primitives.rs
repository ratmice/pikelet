use cgmath::Point3;
use math;

use super::half_edge::*;

pub fn triangle(scale: f32) -> Mesh {
    let extent = scale / 2.0;
    let positions = vec![
        Point3::new(0.0, extent, 0.0),
        Point3::new(-extent, -extent, 0.0),
        Point3::new(extent, -extent, 0.0)
    ];

    let faces = vec![
        Face::new(0)
    ];

    let edges = vec![
        HalfEdge::new_boundary(0, 0, 1),
        HalfEdge::new_boundary(1, 0, 2),
        HalfEdge::new_boundary(2, 0, 0),
    ];

    Mesh {
        positions: positions,
        edges: edges,
        faces: faces
    }
}

pub fn plane(scale: f32) -> Mesh {
    let extent = scale / 2.0;
    let positions = vec![
        Point3::new(-extent, extent, 0.0),
        Point3::new(-extent, -extent, 0.0),
        Point3::new(extent, -extent, 0.0),
        Point3::new(extent, extent, 0.0),
    ];

    let faces = vec![
        Face::new(0),
        Face::new(3)
    ];

    let edges = vec![
        HalfEdge::new_boundary(0, 0, 1),
        HalfEdge::new_boundary(1, 0, 2),
        HalfEdge::new(2, 0, 0, 3),
        HalfEdge::new(0, 1, 4, 2),
        HalfEdge::new_boundary(2, 1, 5),
        HalfEdge::new_boundary(3, 1, 3),
    ];

    Mesh {
        positions: positions,
        edges: edges,
        faces: faces
    }
}

pub fn tetrahedron(scale: f32) -> Mesh {
    let extent = scale / 2.0;

    let positions = vec![
        Point3::new(extent, extent, extent),
        Point3::new(extent, -extent, -extent),
        Point3::new(-extent, extent, -extent),
        Point3::new(-extent, -extent, extent),
    ];

    let faces = vec![
        Face::new(0),
        Face::new(3),
        Face::new(6),
        Face::new(9),
    ];

    let edges = vec![
        // Face 0
        HalfEdge::new(0, 0,  1,  5),
        HalfEdge::new(3, 0,  2, 11),
        HalfEdge::new(1, 0,  0,  6),
        // Face 1
        HalfEdge::new(0, 1,  4,  8),
        HalfEdge::new(1, 1,  5,  9),
        HalfEdge::new(2, 1,  3,  0),
        // Face 2
        HalfEdge::new(0, 2,  7,  2),
        HalfEdge::new(2, 2,  8, 10),
        HalfEdge::new(3, 2,  6,  3),
        // Face 3
        HalfEdge::new(2, 3, 10,  4),
        HalfEdge::new(1, 3, 11,  7),
        HalfEdge::new(3, 3,  9,  1),
    ];

    Mesh {
        positions: positions,
        edges: edges,
        faces: faces
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
    let positions = vec![
        math::set_radius(Point3::new( 0.0,  phi, -1.0), radius),
        math::set_radius(Point3::new(-phi,  1.0,  0.0), radius),
        math::set_radius(Point3::new( 0.0,  phi,  1.0), radius),
        math::set_radius(Point3::new( phi,  1.0,  0.0), radius),
        math::set_radius(Point3::new( 1.0,  0.0, -phi), radius),
        math::set_radius(Point3::new(-1.0,  0.0, -phi), radius),
        math::set_radius(Point3::new(-1.0,  0.0,  phi), radius),
        math::set_radius(Point3::new( 1.0,  0.0,  phi), radius),
        math::set_radius(Point3::new( phi, -1.0,  0.0), radius),
        math::set_radius(Point3::new( 0.0, -phi, -1.0), radius),
        math::set_radius(Point3::new(-phi, -1.0,  0.0), radius),
        math::set_radius(Point3::new( 0.0, -phi,  1.0), radius),
    ];

                       // Edges around
    let faces = vec![  // the face:
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
        HalfEdge::new(       0,    0,    1,  14), // 0
        HalfEdge::new(       1,    0,    2,  17), // 1
        HalfEdge::new(       2,    0,    0,   3), // 2
        // Face 1
        HalfEdge::new(       0,    1,    4,   2), // 3
        HalfEdge::new(       2,    1,    5,  23), // 4
        HalfEdge::new(       3,    1,    3,   6), // 5
        // Face 2
        HalfEdge::new(       0,    2,    7,   5), // 6
        HalfEdge::new(       3,    2,    8,  29), // 7
        HalfEdge::new(       4,    2,    6,   9), // 8
        // Face 3
        HalfEdge::new(       0,    3,   10,   8), // 9
        HalfEdge::new(       4,    3,   11,  35), // 10
        HalfEdge::new(       5,    3,    9,  12), // 11
        // Face 4
        HalfEdge::new(       0,    4,   13,  11), // 12
        HalfEdge::new(       5,    4,   14,  41), // 13
        HalfEdge::new(       1,    4,   12,   0), // 14
        // Face 5
        HalfEdge::new(       1,    5,   16,  44), // 15
        HalfEdge::new(       6,    5,   17,  18), // 16
        HalfEdge::new(       2,    5,   15,   1), // 17
        // Face 6
        HalfEdge::new(       2,    6,   19,  16), // 18
        HalfEdge::new(       7,    6,   20,  47), // 19
        HalfEdge::new(       3,    6,   18,  21), // 20
        // Face 7
        HalfEdge::new(       3,    7,   22,  20), // 21
        HalfEdge::new(       8,    7,   23,  24), // 22
        HalfEdge::new(       4,    7,   21,   4), // 23
        // Face 8
        HalfEdge::new(       4,    8,   25,  22), // 24
        HalfEdge::new(       9,    8,   26,  50), // 25
        HalfEdge::new(       5,    8,   24,  27), // 26
        // Face 9
        HalfEdge::new(       5,    9,   28,  26), // 27
        HalfEdge::new(      10,    9,   29,  30), // 28
        HalfEdge::new(       1,    9,   27,   7), // 29
        // Face 10
        HalfEdge::new(       2,   10,   31,  28), // 30
        HalfEdge::new(       6,   10,   32,  53), // 31
        HalfEdge::new(       7,   10,   30,  33), // 32
        // Face 11
        HalfEdge::new(       3,   11,   34,  32), // 33
        HalfEdge::new(       7,   11,   35,  36), // 34
        HalfEdge::new(       8,   11,   33,  10), // 35
        // Face 12
        HalfEdge::new(       4,   12,   37,  34), // 36
        HalfEdge::new(       8,   12,   38,  56), // 37
        HalfEdge::new(       9,   12,   36,  39), // 38
        // Face 13
        HalfEdge::new(       5,   13,   40,  38), // 39
        HalfEdge::new(       9,   13,   41,  42), // 40
        HalfEdge::new(      10,   13,   39,  13), // 41
        // Face 14
        HalfEdge::new(       1,   14,   43,  40), // 42
        HalfEdge::new(      10,   14,   44,  59), // 43
        HalfEdge::new(       6,   14,   42,  15), // 44
        // Face 15
        HalfEdge::new(       6,   15,   46,  58), // 45
        HalfEdge::new(      11,   15,   47,  48), // 46
        HalfEdge::new(       7,   15,   45,  19), // 47
        // Face 16
        HalfEdge::new(       7,   16,   49,  46), // 48
        HalfEdge::new(      11,   16,   50,  51), // 49
        HalfEdge::new(       8,   16,   48,  25), // 50
        // Face 17
        HalfEdge::new(       8,   17,   52,  49), // 51
        HalfEdge::new(      11,   17,   53,  54), // 52
        HalfEdge::new(       9,   17,   51,  31), // 53
        // Face 18
        HalfEdge::new(       9,   18,   55,  52), // 54
        HalfEdge::new(      11,   18,   56,  57), // 55
        HalfEdge::new(      10,   18,   54,  37), // 56
        // Face 19
        HalfEdge::new(      10,   19,   58,  55), // 57
        HalfEdge::new(      11,   19,   59,  45), // 58
        HalfEdge::new(       6,   19,   57,  43), // 59
    ];

    Mesh {
        positions: positions,
        faces: faces,
        edges: edges
    }
}
