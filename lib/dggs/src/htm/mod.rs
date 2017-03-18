//! The `htm` module contains an implementation of a Descrete Gedesic Grid
//! based on the paper "Navigating through Triangle Meshes Implemented as Linear Quadtrees"
//! written by Michael Lee and Hanan Samet of the University of Maryland.
//! Published in: ACM Transactions on Graphics, Vol. 19, No. 2, April 2000, Pages 79–121.
//!
//! *TODO: Update with notes and summary of the relevant algorithms*
//!

pub mod cell;

pub trait NodeData {
    fn default() -> Self;
}

pub struct Node<T: NodeData> {
    pub path: cell::Path,
    pub orientation: cell::Orientation,
    pub data: T,
}

impl<T: NodeData> Node<T> {
    pub fn new(path: cell::Path, orientation: cell::Orientation, data: T) -> Node<T> {
        Node {
            path: path,
            orientation: orientation,
            data: data,
        }
    }
}

pub struct QuadTree<T: NodeData> {
    pub subdivision_level: cell::Level,
    pub orientation: cell::Orientation,
    pub nodes: Vec<Node<T>>,
}

impl<T: NodeData> QuadTree<T> {
    pub fn with_orientation(orientation: cell::Orientation,
                            subdivision_level: cell::Level)
                            -> QuadTree<T> {
        let cell_count = 4 ^ subdivision_level;

        let mut tree = QuadTree {
            subdivision_level: subdivision_level,
            orientation: orientation,
            nodes: Vec::with_capacity(cell_count as usize),
        };

        for path in (0..cell_count).map(cell::Path) {
            let cell_orientation = path.orientation(subdivision_level, orientation);
            tree.nodes.push(Node::new(path, cell_orientation, T::default()));
        }

        tree
    }
}

pub struct Icosahedron<T: NodeData> {
    nodes: Vec<QuadTree<T>>,
}

impl<T: NodeData> Icosahedron<T> {
    pub fn with_subdivisions(subdivision_level: cell::Level) -> Icosahedron<T> {
        Icosahedron {
            nodes: vec![QuadTree::with_orientation(cell::Orientation::Up, subdivision_level), // 0
                        QuadTree::with_orientation(cell::Orientation::Up, subdivision_level), // 1
                        QuadTree::with_orientation(cell::Orientation::Up, subdivision_level), // 2
                        QuadTree::with_orientation(cell::Orientation::Up, subdivision_level), // 3
                        QuadTree::with_orientation(cell::Orientation::Up, subdivision_level), // 4
                        QuadTree::with_orientation(cell::Orientation::Down, subdivision_level), // 5
                        QuadTree::with_orientation(cell::Orientation::Down, subdivision_level), // 6
                        QuadTree::with_orientation(cell::Orientation::Down, subdivision_level), // 7
                        QuadTree::with_orientation(cell::Orientation::Down, subdivision_level), // 8
                        QuadTree::with_orientation(cell::Orientation::Down, subdivision_level), // 9
                        QuadTree::with_orientation(cell::Orientation::Up, subdivision_level), // 10
                        QuadTree::with_orientation(cell::Orientation::Up, subdivision_level), // 11
                        QuadTree::with_orientation(cell::Orientation::Up, subdivision_level), // 12
                        QuadTree::with_orientation(cell::Orientation::Up, subdivision_level), // 13
                        QuadTree::with_orientation(cell::Orientation::Up, subdivision_level), // 14
                        QuadTree::with_orientation(cell::Orientation::Down, subdivision_level), // 15
                        QuadTree::with_orientation(cell::Orientation::Down, subdivision_level), // 16
                        QuadTree::with_orientation(cell::Orientation::Down, subdivision_level), // 17
                        QuadTree::with_orientation(cell::Orientation::Down, subdivision_level), // 18
                        QuadTree::with_orientation(cell::Orientation::Down, subdivision_level)], // 19
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Tests
////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::QuadTree;
    use super::cell;

    struct TestData;

    impl super::NodeData for TestData {
        fn default() -> TestData {
            TestData {}
        }
    }

    type TestTree = QuadTree<TestData>;

    fn tipup_quadtree(subdivision_level: cell::Level) -> TestTree {
        QuadTree::with_orientation(cell::Orientation::Up, subdivision_level)
    }

    fn tipdown_quadtree(subdivision_level: cell::Level) -> TestTree {
        QuadTree::with_orientation(cell::Orientation::Down, subdivision_level)
    }

    fn assert_orientations(start: cell::Index, tree: &TestTree) {
        for offset in 0..3 {
            let index = start + offset;
            let cell_orientation = &tree.nodes[index as usize].orientation;
            let expected_orientation = match tree.orientation {
                orientation if offset != 2 => orientation,
                cell::Orientation::Up => cell::Orientation::Down,
                cell::Orientation::Down => cell::Orientation::Up,
            };

            println!("Path: {}, Expected: {:?}, Actual: {:?}",
                     index,
                     expected_orientation,
                     cell_orientation);
            assert_eq!(expected_orientation, *cell_orientation);
        }
    }

    #[test]
    fn shallow_tipup_quadtree_fundamentals() {
        let subdivision_level = 1;
        let qt_up = tipup_quadtree(subdivision_level);
        assert_orientations(0, &qt_up);
    }

    #[test]
    fn shallow_tipdown_quadtree_fundamentals() {
        let subdivision_level = 1;
        let qt_down = tipdown_quadtree(subdivision_level);
        assert_orientations(0, &qt_down);
    }

    #[test]
    fn deep_tipup_quadtree_fundamentals() {
        let subdivision_level = 3;
        let qt_up = tipup_quadtree(subdivision_level);
        let mut root_index = 0;
        let node_count = subdivision_level ^ 4;
        while root_index < node_count {
            assert_orientations(root_index, &qt_up);
            root_index += 4;
        }
    }

    #[test]
    fn icosahedron_fundamentals() {
        unimplemented!();
    }

}
