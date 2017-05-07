use cgmath::Point3;

use super::*;
use EdgeIndex as Ei;
use FaceIndex as Fi;
use PositionIndex as Pi;

pub fn triangle(scale: f32) -> Mesh<Point3<f32>> {
    let extent = scale / 2.0;
    let positions = vec![Point3::new(0.0, extent, 0.0),
                         Point3::new(-extent, -extent, 0.0),
                         Point3::new(extent, -extent, 0.0)];

    let faces = vec![Face::new(Ei(0))];

    let edges = vec![Edge::new_boundary(Pi(0), Fi(0), Ei(1)),
                     Edge::new_boundary(Pi(1), Fi(0), Ei(2)),
                     Edge::new_boundary(Pi(2), Fi(0), Ei(0))];

    Mesh {
        positions: positions,
        edges: edges,
        faces: faces,
    }
}

pub fn plane(scale: f32) -> Mesh<Point3<f32>> {
    let extent = scale / 2.0;
    let positions = vec![Point3::new(-extent, extent, 0.0),
                         Point3::new(-extent, -extent, 0.0),
                         Point3::new(extent, -extent, 0.0),
                         Point3::new(extent, extent, 0.0)];

    let faces = vec![Face::new(Ei(0)), Face::new(Ei(3))];

    let edges = vec![Edge::new_boundary(Pi(0), Fi(0), Ei(1)),
                     Edge::new_boundary(Pi(1), Fi(0), Ei(2)),
                     Edge::new(Pi(2), Fi(0), Ei(0), Ei(3)),
                     Edge::new(Pi(0), Fi(1), Ei(4), Ei(2)),
                     Edge::new_boundary(Pi(2), Fi(1), Ei(5)),
                     Edge::new_boundary(Pi(3), Fi(1), Ei(3))];

    Mesh {
        positions: positions,
        edges: edges,
        faces: faces,
    }
}

pub fn tetrahedron(scale: f32) -> Mesh<Point3<f32>> {
    let extent = scale / 2.0;

    let positions = vec![Point3::new(extent, extent, extent),
                         Point3::new(extent, -extent, -extent),
                         Point3::new(-extent, extent, -extent),
                         Point3::new(-extent, -extent, extent)];

    let faces = vec![Face::new(Ei(0)),
                     Face::new(Ei(3)),
                     Face::new(Ei(6)),
                     Face::new(Ei(9))];

    let edges = vec![
        //        point,  face,   next,     adj,
        // Face 0
        Edge::new(Pi(0), Fi(0), Ei( 1), Ei( 8)), //  0
        Edge::new(Pi(3), Fi(0), Ei( 2), Ei(10)), //  1
        Edge::new(Pi(1), Fi(0), Ei( 0), Ei( 3)), //  2
        // Face 1
        Edge::new(Pi(0), Fi(1), Ei( 4), Ei( 2)), //  3
        Edge::new(Pi(1), Fi(1), Ei( 5), Ei( 9)), //  4
        Edge::new(Pi(2), Fi(1), Ei( 3), Ei( 6)), //  5
        // Face 2
        Edge::new(Pi(0), Fi(2), Ei( 7), Ei( 5)), //  6
        Edge::new(Pi(2), Fi(2), Ei( 8), Ei(11)), //  7
        Edge::new(Pi(3), Fi(2), Ei( 6), Ei( 0)), //  8
        // Face 3
        Edge::new(Pi(2), Fi(3), Ei(10), Ei( 4)), //  9
        Edge::new(Pi(1), Fi(3), Ei(11), Ei( 1)), // 10
        Edge::new(Pi(3), Fi(3), Ei( 9), Ei( 7)), // 11
    ];

    Mesh {
        positions: positions,
        edges: edges,
        faces: faces,
    }
}

pub fn icosahedron(radius: f32) -> Mesh<Point3<f32>> {
    // The coordinates of the iocosahedron are are described by the
    // cyclic permutations of (±ϕ, ±1, 0), where ϕ is the [Golden Ratio]
    // (https://en.wikipedia.org/wiki/Golden_ratio).
    // NOTE: The order here is such that 0 - 11 line up with the logical
    //       winding of faces over the "net" of the polyhedron.
    //       (would be cool if you could annotate source code with images
    //        to explain notes like this!)
    let phi = (1.0 + f32::sqrt(5.0)) / 2.0;
    let positions = vec![set_radius(Point3::new(0.0, phi, -1.0), radius),
                         set_radius(Point3::new(-phi, 1.0, 0.0), radius),
                         set_radius(Point3::new(0.0, phi, 1.0), radius),
                         set_radius(Point3::new(phi, 1.0, 0.0), radius),
                         set_radius(Point3::new(1.0, 0.0, -phi), radius),
                         set_radius(Point3::new(-1.0, 0.0, -phi), radius),
                         set_radius(Point3::new(-1.0, 0.0, phi), radius),
                         set_radius(Point3::new(1.0, 0.0, phi), radius),
                         set_radius(Point3::new(phi, -1.0, 0.0), radius),
                         set_radius(Point3::new(0.0, -phi, -1.0), radius),
                         set_radius(Point3::new(-phi, -1.0, 0.0), radius),
                         set_radius(Point3::new(0.0, -phi, 1.0), radius)];

    // Edges around the face:
    let faces = vec![
        Face::new(Ei( 0)), //  0,  1,  2
        Face::new(Ei( 3)), //  3,  4,  5
        Face::new(Ei( 6)), //  6,  7,  8
        Face::new(Ei( 9)), //  9, 10, 11
        Face::new(Ei(12)), // 12, 13, 14
        Face::new(Ei(15)), // 15, 16, 17
        Face::new(Ei(18)), // 18, 19, 20
        Face::new(Ei(21)), // 21, 22, 23
        Face::new(Ei(24)), // 24, 25, 26
        Face::new(Ei(27)), // 27, 28, 29
        Face::new(Ei(30)), // 30, 31, 32
        Face::new(Ei(33)), // 33, 34, 35
        Face::new(Ei(36)), // 36, 37, 38
        Face::new(Ei(39)), // 39, 40, 41
        Face::new(Ei(42)), // 42, 43, 44
        Face::new(Ei(45)), // 45, 46, 47
        Face::new(Ei(48)), // 48, 49, 50
        Face::new(Ei(51)), // 51, 52, 53
        Face::new(Ei(54)), // 54, 55, 56
        Face::new(Ei(57)), // 57, 58, 59
    ];

    let edges = vec![
        //         point, face,     next,     adj,
        // Face 0
        Edge::new(Pi( 0), Fi( 0), Ei( 1), Ei(14)), // 0
        Edge::new(Pi( 1), Fi( 0), Ei( 2), Ei(17)), // 1
        Edge::new(Pi( 2), Fi( 0), Ei( 0), Ei( 3)), // 2
        // Face 1
        Edge::new(Pi( 0), Fi( 1), Ei( 4), Ei( 2)), // 3
        Edge::new(Pi( 2), Fi( 1), Ei( 5), Ei(20)), // 4
        Edge::new(Pi( 3), Fi( 1), Ei( 3), Ei( 6)), // 5
        // Face 2
        Edge::new(Pi( 0), Fi( 2), Ei( 7), Ei( 5)), // 6
        Edge::new(Pi( 3), Fi( 2), Ei( 8), Ei(23)), // 7
        Edge::new(Pi( 4), Fi( 2), Ei( 6), Ei( 9)), // 8
        // Face 3
        Edge::new(Pi( 0), Fi( 3), Ei(10), Ei( 8)), // 9
        Edge::new(Pi( 4), Fi( 3), Ei(11), Ei(26)), // 10
        Edge::new(Pi( 5), Fi( 3), Ei( 9), Ei(12)), // 11
        // Face 4
        Edge::new(Pi( 0), Fi( 4), Ei(13), Ei(11)), // 12
        Edge::new(Pi( 5), Fi( 4), Ei(14), Ei(29)), // 13
        Edge::new(Pi( 1), Fi( 4), Ei(12), Ei( 0)), // 14
        // Face 5
        Edge::new(Pi( 1), Fi( 5), Ei(16), Ei(44)), // 15
        Edge::new(Pi( 6), Fi( 5), Ei(17), Ei(30)), // 16
        Edge::new(Pi( 2), Fi( 5), Ei(15), Ei( 1)), // 17
        // Face 6
        Edge::new(Pi( 2), Fi( 6), Ei(19), Ei(32)), // 18
        Edge::new(Pi( 7), Fi( 6), Ei(20), Ei(33)), // 19
        Edge::new(Pi( 3), Fi( 6), Ei(18), Ei( 4)), // 20
        // Face 7
        Edge::new(Pi( 3), Fi( 7), Ei(22), Ei(35)), // 21
        Edge::new(Pi( 8), Fi( 7), Ei(23), Ei(36)), // 22
        Edge::new(Pi( 4), Fi( 7), Ei(21), Ei( 7)), // 23
        // Face 8
        Edge::new(Pi( 4), Fi( 8), Ei(25), Ei(38)), // 24
        Edge::new(Pi( 9), Fi( 8), Ei(26), Ei(39)), // 25
        Edge::new(Pi( 5), Fi( 8), Ei(24), Ei(10)), // 26
        // Face 9
        Edge::new(Pi( 5), Fi( 9), Ei(28), Ei(41)), // 27
        Edge::new(Pi(10), Fi( 9), Ei(29), Ei(42)), // 28
        Edge::new(Pi( 1), Fi( 9), Ei(27), Ei(13)), // 29
        // Face 10
        Edge::new(Pi( 2), Fi(10), Ei(31), Ei(16)), // 30
        Edge::new(Pi( 6), Fi(10), Ei(32), Ei(47)), // 31
        Edge::new(Pi( 7), Fi(10), Ei(30), Ei(18)), // 32
        // Face 11
        Edge::new(Pi( 3), Fi(11), Ei(34), Ei(19)), // 33
        Edge::new(Pi( 7), Fi(11), Ei(35), Ei(50)), // 34
        Edge::new(Pi( 8), Fi(11), Ei(33), Ei(21)), // 35
        // Face 12
        Edge::new(Pi( 4), Fi(12), Ei(37), Ei(22)), // 36
        Edge::new(Pi( 8), Fi(12), Ei(38), Ei(53)), // 37
        Edge::new(Pi( 9), Fi(12), Ei(36), Ei(24)), // 38
        // Face 13
        Edge::new(Pi( 5), Fi(13), Ei(40), Ei(25)), // 39
        Edge::new(Pi( 9), Fi(13), Ei(41), Ei(56)), // 40
        Edge::new(Pi(10), Fi(13), Ei(39), Ei(27)), // 41
        // Face 14
        Edge::new(Pi( 1), Fi(14), Ei(43), Ei(28)), // 42
        Edge::new(Pi(10), Fi(14), Ei(44), Ei(59)), // 43
        Edge::new(Pi( 6), Fi(14), Ei(42), Ei(15)), // 44
        // Face 15
        Edge::new(Pi( 6), Fi(15), Ei(46), Ei(58)), // 45
        Edge::new(Pi(11), Fi(15), Ei(47), Ei(48)), // 46
        Edge::new(Pi( 7), Fi(15), Ei(45), Ei(31)), // 47
        // Face 16
        Edge::new(Pi( 7), Fi(16), Ei(49), Ei(46)), // 48
        Edge::new(Pi(11), Fi(16), Ei(50), Ei(51)), // 49
        Edge::new(Pi( 8), Fi(16), Ei(48), Ei(34)), // 50
        // Face 17
        Edge::new(Pi( 8), Fi(17), Ei(52), Ei(49)), // 51
        Edge::new(Pi(11), Fi(17), Ei(53), Ei(54)), // 52
        Edge::new(Pi( 9), Fi(17), Ei(51), Ei(37)), // 53
        // Face 18
        Edge::new(Pi( 9), Fi(18), Ei(55), Ei(52)), // 54
        Edge::new(Pi(11), Fi(18), Ei(56), Ei(57)), // 55
        Edge::new(Pi(10), Fi(18), Ei(54), Ei(40)), // 56
        // Face 19
        Edge::new(Pi(10), Fi(19), Ei(58), Ei(55)), // 57
        Edge::new(Pi(11), Fi(19), Ei(59), Ei(45)), // 58
        Edge::new(Pi( 6), Fi(19), Ei(57), Ei(43)), // 59
    ];

    Mesh {
        positions: positions,
        faces: faces,
        edges: edges,
    }
}
