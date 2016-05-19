pub mod half_edge;
pub mod primitives;
pub mod star_field;

#[cfg(test)]
mod tests {
    use ::math;
    use super::primitives;
    use super::half_edge;

    fn assert_congruent_adjacenct_positions(e0: &half_edge::HalfEdge, e1: &half_edge::HalfEdge,
                                            mesh: &half_edge::Mesh) {
        let e0p0 = e0.position.clone();
        let e0p1 = mesh.edges[e0.next].position.clone();

        let e1p0 = e1.position.clone();
        let e1p1 = mesh.edges[e1.next].position.clone();

        assert_eq!(e0p0, e1p1);
        assert_eq!(e0p1, e1p0);
    }

    fn assert_congruent_adjacency(index: &half_edge::EdgeIndex, edge: &half_edge::HalfEdge,
                                  mesh: &half_edge::Mesh) {
        let adjacent_index = edge.adjacent.unwrap().clone();
        let ref adjacent_edge = mesh.edges[adjacent_index];
        assert!(adjacent_edge.adjacent.is_some());
        
        let expected_index = adjacent_edge.adjacent.unwrap().clone();
        assert_eq!(*index, expected_index);
        
        assert_congruent_adjacenct_positions(&edge, &adjacent_edge, &mesh);
    }

    // used to test meshes that should have no boundary edges
    fn assert_congruent_nonboundary_mesh(mesh: &half_edge::Mesh) {
        for (index, edge) in mesh.edges.iter().enumerate() {
            assert!(edge.adjacent.is_some());
            assert_congruent_adjacency(&index, &edge, &mesh);
        }
    }

    // used to test meshes which are allowed to have boundary edges
    fn assert_congruent_mesh(mesh: &half_edge::Mesh) {
        for (index, edge) in mesh.edges.iter().enumerate() {
            if edge.adjacent.is_none() {
                continue
            }
            assert_congruent_adjacency(&index, &edge, &mesh);
        }
    }

    #[test]
    fn icosahedron() {
        let planet_radius: f32 = 1.0;
        let icosahedron = primitives::icosahedron(planet_radius);
        assert_congruent_nonboundary_mesh(&icosahedron);
    }

    #[test]
    fn tetrahedron() {
        let scale: f32 = 1.0;
        let mesh = primitives::tetrahedron(scale);
        assert_congruent_nonboundary_mesh(&mesh);
    }

    #[test]
    fn plane() {
        let scale: f32 = 1.0;
        let plane = primitives::plane(scale);
        assert_congruent_mesh(&plane);
    }

    #[test]
    fn triangle() {
        let scale: f32 = 1.0;
        let mesh = primitives::triangle(scale);
        assert_congruent_mesh(&mesh);
    }

    #[test]
    fn subdivided_icosahedron() {
        let subdivisions: usize = 3;
        let planet_radius: f32 = 1.0;

        let icosahedron = primitives::icosahedron(planet_radius);
        let mesh = icosahedron.subdivide(subdivisions, &|a, b| {
                math::midpoint_arc(planet_radius, a, b)
            });
        assert_congruent_nonboundary_mesh(&mesh);
    }

    #[test]
    fn subdivided_tetrahedron() {
        let subdivisions: usize = 3;
        let scale: f32 = 1.0;

        let tetrahedron = primitives::tetrahedron(scale);
        let mesh = tetrahedron.subdivide(subdivisions, &|a, b| {
                math::midpoint(a, b)
            });
        assert_congruent_nonboundary_mesh(&mesh);
    }

    #[test]
    fn subdivided_triangle() {
        let subdivisions: usize = 3;
        let scale: f32 = 1.0;

        let tri = primitives::triangle(scale);
        let mesh = tri.subdivide(subdivisions, &|a, b| {
                math::midpoint(a, b)
            });
        assert_congruent_mesh(&mesh);
    }
    
    #[test]
    fn subdivided_plane() {
        let subdivisions: usize = 3;
        let scale: f32 = 1.0;

        let plane = primitives::plane(scale);
        let mesh = plane.subdivide(subdivisions, &|a, b| {
                math::midpoint(a, b)
            });
        assert_congruent_mesh(&mesh);
    }
}
