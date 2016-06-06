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

    #[derive(Clone, Debug)]
    pub enum Orientation {
        TipUp,
        TipDown
    }

    impl fmt::Display for Orientation {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                Orientation::TipUp => write!(f, "Tip-Up"),
                Orientation::TipDown => write!(f, "Tip-Down"),
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
        fn to_bits(&self) -> u32 {
            match *self {
                Id::Top         => 0b00,
                Id::Center      => 0b10,
                Id::TopLeft     => 0b01,
                Id::TopRight    => 0b11,
                Id::Bottom      => 0b00,
                Id::BottomLeft  => 0b01,
                Id::BottomRight => 0b11,
            }
        }

        fn from_bits(orientation: Orientation, bits: u32) -> Id {
            match orientation {
                Orientation::TipUp => match bits {
                    0b00 => Id::Top,
                    0b10 => Id::Center,
                    0b01 => Id::BottomLeft,
                    0b11 => Id::BottomRight,
                    _ => panic!("Not a valid CellId value."),
                },
                Orientation::TipDown => match bits {
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
        level: usize,
        path: usize,
    }

    impl fmt::Display for Location {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "LOC:{},{}", self.level, self.path)
        }
    }

    pub struct Data {
        orientation: Orientation,
        location: Location,
    }

    impl Data {
        pub fn default() -> Data {
            Data {
                orientation: Orientation::TipUp,
                location: Location {
                    level: 0,
                    path: 0,
                }
            }
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
    nodes: Vec<cell::Data>,
}

impl QuadTree {
    pub fn with_orientation(orientation: cell::Orientation, levels: usize) -> QuadTree {
        let tree = QuadTree {
            nodes: Vec::with_capacity(levels * 4),
        };

        tree
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
                QuadTree::with_orientation(cell::Orientation::TipUp, levels),
                // 1
                QuadTree::with_orientation(cell::Orientation::TipUp, levels),
                // 2
                QuadTree::with_orientation(cell::Orientation::TipUp, levels),
                // 3
                QuadTree::with_orientation(cell::Orientation::TipUp, levels),
                // 4
                QuadTree::with_orientation(cell::Orientation::TipUp, levels),
                // 5
                QuadTree::with_orientation(cell::Orientation::TipDown, levels),
                // 6
                QuadTree::with_orientation(cell::Orientation::TipDown, levels),
                // 7
                QuadTree::with_orientation(cell::Orientation::TipDown, levels),
                // 8
                QuadTree::with_orientation(cell::Orientation::TipDown, levels),
                // 9
                QuadTree::with_orientation(cell::Orientation::TipDown, levels),
                // 10
                QuadTree::with_orientation(cell::Orientation::TipUp, levels),
                // 11
                QuadTree::with_orientation(cell::Orientation::TipUp, levels),
                // 12
                QuadTree::with_orientation(cell::Orientation::TipUp, levels),
                // 13
                QuadTree::with_orientation(cell::Orientation::TipUp, levels),
                // 14
                QuadTree::with_orientation(cell::Orientation::TipUp, levels),
                // 15
                QuadTree::with_orientation(cell::Orientation::TipDown, levels),
                // 16
                QuadTree::with_orientation(cell::Orientation::TipDown, levels),
                // 17
                QuadTree::with_orientation(cell::Orientation::TipDown, levels),
                // 18
                QuadTree::with_orientation(cell::Orientation::TipDown, levels),
                // 19
                QuadTree::with_orientation(cell::Orientation::TipDown, levels),
            ]
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Tests
////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
}
