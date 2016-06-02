//! The `htm` module contains an implementation of a Descrete Gedesic Grid
//! based on the paper "Navigating through Triangle Meshes Implemented as Linear Quadtrees"
//! written by Michael Lee and Hanan Samet of the University of Maryland.
//! Published in: ACM Transactions on Graphics, Vol. 19, No. 2, April 2000, Pages 79–121.
//!
//! *TODO: Update with notes and summary of the relevant algorithms*
//!

pub mod cell {
    pub type Index = usize;

    #[derive(Clone, Copy, Debug)]
    pub enum Orientation {
        TipUp,
        TipDown
    }

    #[derive(Clone, Copy, Debug)]
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

    #[derive(Clone, Copy, Debug)]
    pub struct Location {
        level: usize,
        path: usize,
    }

    #[derive(Clone, Copy, Debug)]
    pub struct Data {
        orientation: Orientation,
        location: Location,
    }
}

#[derive(Clone, Debug)]
pub struct QuadTree {
    nodes: Vec<cell::Data>,
}

impl QuadTree {
    pub fn with_subdivisions(levels: usize) -> QuadTree {
        let nodes = Vec::with_capacity(levels);

        QuadTree {
            nodes: nodes
        }
    }
}

pub struct Icosahedron {
    nodes: [QuadTree; 20],
}

impl Icosahedron {
    pub fn with_subdivisions(levels: usize) -> Icosahedron {
        Icosahedron {
            nodes: [QuadTree::with_subdivisions(levels); 20]
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Tests
////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
}
