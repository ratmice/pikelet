pub mod half_edge;
pub mod primitives;
pub mod star_field;

#[cfg(test)]
mod tests {
    use ::math;
    use super::primitives;
    use super::half_edge;

    #[test]
    fn subdivide() {
        let subdivisions: usize = 3;
        let planet_radius: f32 = 1.0;
        
        let mesh = primitives::icosahedron(planet_radius)
            .subdivide(subdivisions, &|a, b| {
                math::midpoint_arc(planet_radius, a, b)
            });
        for (index, edge) in mesh.edges.iter().enumerate() {
            assert!(edge.adjacent.is_some());
            let adjacent_index = edge.adjacent.unwrap().clone();
            let expected_index = mesh.edges[adjacent_index].adjacent.unwrap().clone();
            assert_eq!(index, expected_index);
        }
    }
}
