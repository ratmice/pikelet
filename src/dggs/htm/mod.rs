//! The `htm` module contains an implementation of a Descrete Gedesic Grid
//! based on the paper "Navigating through Triangle Meshes Implemented as Linear Quadtrees"
//! written by Michael Lee and Hanan Samet of the University of Maryland.
//! Published in: ACM Transactions on Graphics, Vol. 19, No. 2, April 2000, Pages 79–121.
//!
//! *TODO: Update with notes and summary of the relevant algorithms*
//!

pub mod cell {
    use std::fmt;

    pub type Index = usize;
    pub type Level = usize;
    pub type Path = usize;

    #[derive(Clone, Debug)]
    pub enum Orientation {
        Up,
        Down
    }

    impl fmt::Display for Orientation {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                Orientation::Up => write!(f, "Up"),
                Orientation::Down => write!(f, "Down"),
            }
        }
    }

    #[derive(Clone, Debug)]
    pub enum Id {
        Top,
        Center,
        TopLeft,
        TopRight,
        Bottom,
        BottomLeft,
        BottomRight
    }

    impl Id {
        #[cfg_attr(feature = "clippy", allow(match_same_arms))]
        pub fn to_bits(id: Id) -> u32 {
            match id {
                Id::Top         => 0b00,
                Id::Center      => 0b10,
                Id::TopLeft     => 0b01,
                Id::TopRight    => 0b11,
                Id::Bottom      => 0b00,
                Id::BottomLeft  => 0b01,
                Id::BottomRight => 0b11,
            }
        }

        pub fn from_bits(orientation: Orientation, bits: u32) -> Id {
            match orientation {
                Orientation::Up => match bits {
                    0b00 => Id::Top,
                    0b10 => Id::Center,
                    0b01 => Id::BottomLeft,
                    0b11 => Id::BottomRight,
                    _ => panic!("Not a valid CellId value."),
                },
                Orientation::Down => match bits {
                    0b00 => Id::Bottom,
                    0b10 => Id::Center,
                    0b01 => Id::TopLeft,
                    0b11 => Id::TopRight,
                    _ => panic!("Not a valid CellId value."),
                }
            }
        }
    }

    #[derive(Clone, Debug)]
    pub struct Location {
        level: Level,
        path: Path,
    }

    impl fmt::Display for Location {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{{ {},{} }}", self.level, self.path)
        }
    }

    pub struct Data {
        orientation: Orientation,
        location: Location,
    }

    impl Data {
        pub fn new(orientation: Orientation, location: Location) -> Data {
            Data {
                orientation: orientation,
                location: location
            }
        }

        pub fn default() -> Data {
            Data::new(
                Orientation::Up,
                Location {
                    level: 0,
                    path: 0,
                }
            )
        }
    }

    impl fmt::Debug for Data {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "cell::Data {{ orientation: {}, location: {} }}",
                   self.orientation, self.location)
        }
    }
}

pub struct QuadTree {
    levels: cell::Level,
    orientation: cell::Orientation,
    nodes: Vec<cell::Data>,
}

impl QuadTree {
    pub fn with_orientation(orientation: cell::Orientation, levels: cell::Level) -> QuadTree {
        let cell_count = 4 ^ levels;

        let mut tree = QuadTree {
            levels: levels,
            orientation: orientation,
            nodes: Vec::with_capacity(cell_count),
        };

        for i in 0..cell_count {
            // `i` here is essentially the full path to the current cell at the fully
            // subdivided level. This means that we can determine the correct attributes
            // in terms of path and orientation by analyzing this number in relation to
            // the overall orientation of the quadtree.
            tree.nodes.push(cell::Data::default());
        }

        tree
    }

    pub fn location_for(id: cell::Id, orientation: cell::Orientation, level: cell::Level) -> cell::Location {
        unimplemented!()
    }
}

pub struct Icosahedron {
    nodes: Vec<QuadTree>,
}

impl Icosahedron {
    pub fn with_subdivisions(levels: usize) -> Icosahedron {
        Icosahedron {
            nodes: vec! [
                // 0
                QuadTree::with_orientation(cell::Orientation::Up, levels),
                // 1
                QuadTree::with_orientation(cell::Orientation::Up, levels),
                // 2
                QuadTree::with_orientation(cell::Orientation::Up, levels),
                // 3
                QuadTree::with_orientation(cell::Orientation::Up, levels),
                // 4
                QuadTree::with_orientation(cell::Orientation::Up, levels),
                // 5
                QuadTree::with_orientation(cell::Orientation::Down, levels),
                // 6
                QuadTree::with_orientation(cell::Orientation::Down, levels),
                // 7
                QuadTree::with_orientation(cell::Orientation::Down, levels),
                // 8
                QuadTree::with_orientation(cell::Orientation::Down, levels),
                // 9
                QuadTree::with_orientation(cell::Orientation::Down, levels),
                // 10
                QuadTree::with_orientation(cell::Orientation::Up, levels),
                // 11
                QuadTree::with_orientation(cell::Orientation::Up, levels),
                // 12
                QuadTree::with_orientation(cell::Orientation::Up, levels),
                // 13
                QuadTree::with_orientation(cell::Orientation::Up, levels),
                // 14
                QuadTree::with_orientation(cell::Orientation::Up, levels),
                // 15
                QuadTree::with_orientation(cell::Orientation::Down, levels),
                // 16
                QuadTree::with_orientation(cell::Orientation::Down, levels),
                // 17
                QuadTree::with_orientation(cell::Orientation::Down, levels),
                // 18
                QuadTree::with_orientation(cell::Orientation::Down, levels),
                // 19
                QuadTree::with_orientation(cell::Orientation::Down, levels),
            ]
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Tests
////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {

    #[test]
    fn zcurve_fundamentals_are_understood() {
        unimplemented!()
    }
}
