//! The `htm` module contains an implementation of a Descrete Gedesic Grid
//! based on the paper "Navigating through Triangle Meshes Implemented as Linear Quadtrees"
//! written by Michael Lee and Hanan Samet of the University of Maryland.
//! Published in: ACM Transactions on Graphics, Vol. 19, No. 2, April 2000, Pages 79–121.
//!
//! *TODO: Update with notes and summary of the relevant algorithms*
//!

pub mod cell {
    pub type Index = usize;
    pub type Level = usize;
    pub type Path = usize;

    pub const ID_MASK: usize = 0x03;
    pub const EVEN_BITS: usize = 0xAAAAAAAAAAAAAAAA;
    pub const ODD_BITS: usize  = 0x5555555555555555;

    pub const CENTER: usize = 0b10;

    pub const TOP: usize = 0b00;
    pub const TOP_LEFT: usize = 0b01;
    pub const TOP_RIGHT: usize = 0b11;

    pub const BOTTOM: usize = 0b00;
    pub const BOTTOM_LEFT: usize = 0b01;
    pub const BOTTOM_RIGHT: usize = 0b11;

    #[derive(Clone, Debug, Copy, PartialEq)]
    pub enum Orientation {
        Up,
        Down
    }

    pub fn orientation_for_path(subdivision_level: Level, tree_orientation: Orientation, path: Path) -> Orientation {
        let node_id = path & ID_MASK;
        let parent_orientation = if subdivision_level == 1 {
            tree_orientation
        } else {
            let parent_path = path >> 2;
            let parent_level = subdivision_level - 1;
            orientation_for_path(parent_level, tree_orientation, parent_path)
        };
        match node_id {
            CENTER => match parent_orientation {
                Orientation::Up => Orientation::Down,
                Orientation::Down => Orientation::Up
            },
            _ => parent_orientation
        }
    }
}

// so many ??? regarding memory fragmenation
pub enum NodeData {
    Empty,
    Index(usize),
    Released(usize)
}

pub struct Node {
    pub path: cell::Path,
    // It may be that we don't have to store this for every node.
    // Since we keep the tree orientation in the QuadTree struct,
    // we'll more or less always have a way to determine cell
    // orientation via it's path. I was just assuming for the moment
    // that a bit of pre-calculation could make life easier down
    // the road.
    pub orientation: cell::Orientation,
    pub data: NodeData
}

impl Node {
    pub fn new(path: cell::Path, orientation: cell::Orientation) -> Node {
        Node {
            path: path,
            orientation: orientation,
            data: NodeData::Empty
        }
    }
}

pub struct QuadTree<T> {
    pub subdivision_level: cell::Level,
    pub orientation: cell::Orientation,
    pub nodes: Vec<Node>,
    data: Vec<T>
}

impl<T> QuadTree<T> {
    pub fn with_orientation(orientation: cell::Orientation, subdivision_level: cell::Level) -> QuadTree<T> {
        let cell_count = 4 ^ subdivision_level;

        let mut tree = QuadTree {
            subdivision_level: subdivision_level,
            orientation: orientation,
            nodes: Vec::with_capacity(cell_count),
            data: Vec::new()
        };

        for path in 0..cell_count {
            let cell_orientation = cell::orientation_for_path(subdivision_level, orientation, path);
            tree.nodes.push(Node::new(path, cell_orientation));
        }

        tree
    }
}

pub struct Icosahedron<T> {
    nodes: Vec<QuadTree<T>>,
}

impl<T> Icosahedron<T> {
    pub fn with_subdivisions(subdivision_level: usize) -> Icosahedron<T> {
        Icosahedron {
            nodes: vec! [
                // 0
                QuadTree::with_orientation(cell::Orientation::Up, subdivision_level),
                // 1
                QuadTree::with_orientation(cell::Orientation::Up, subdivision_level),
                // 2
                QuadTree::with_orientation(cell::Orientation::Up, subdivision_level),
                // 3
                QuadTree::with_orientation(cell::Orientation::Up, subdivision_level),
                // 4
                QuadTree::with_orientation(cell::Orientation::Up, subdivision_level),
                // 5
                QuadTree::with_orientation(cell::Orientation::Down, subdivision_level),
                // 6
                QuadTree::with_orientation(cell::Orientation::Down, subdivision_level),
                // 7
                QuadTree::with_orientation(cell::Orientation::Down, subdivision_level),
                // 8
                QuadTree::with_orientation(cell::Orientation::Down, subdivision_level),
                // 9
                QuadTree::with_orientation(cell::Orientation::Down, subdivision_level),
                // 10
                QuadTree::with_orientation(cell::Orientation::Up, subdivision_level),
                // 11
                QuadTree::with_orientation(cell::Orientation::Up, subdivision_level),
                // 12
                QuadTree::with_orientation(cell::Orientation::Up, subdivision_level),
                // 13
                QuadTree::with_orientation(cell::Orientation::Up, subdivision_level),
                // 14
                QuadTree::with_orientation(cell::Orientation::Up, subdivision_level),
                // 15
                QuadTree::with_orientation(cell::Orientation::Down, subdivision_level),
                // 16
                QuadTree::with_orientation(cell::Orientation::Down, subdivision_level),
                // 17
                QuadTree::with_orientation(cell::Orientation::Down, subdivision_level),
                // 18
                QuadTree::with_orientation(cell::Orientation::Down, subdivision_level),
                // 19
                QuadTree::with_orientation(cell::Orientation::Down, subdivision_level),
            ]
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
            let cell_orientation = &tree.nodes[index].orientation;
            let expected_orientation =
                match tree.orientation {
                    orientation if offset != 2 => orientation,
                    cell::Orientation::Up => cell::Orientation::Down,
                    cell::Orientation::Down => cell::Orientation::Up,
                };

            println!("Path: {}, Expected: {:?}, Actual: {:?}", index, expected_orientation, cell_orientation);
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
