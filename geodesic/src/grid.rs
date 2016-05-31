//! The `grid` module contains an implementation of a Descrete Gedesic Grid
//! based on the paper "Navigating through Triangle Meshes Implemented as Linear Quadtrees"
//! written by Michael Lee and Hanan Samet of the University of Maryland.
//! Published in: ACM Transactions on Graphics, Vol. 19, No. 2, April 2000, Pages 79–121.
//!
//! *TODO: Update with notes and summary of the relevant algorithms*
//!

pub mod cell {
    pub enum Orientation {
        TipUp,
        TipDown
    }

    pub enum ID {
        Top,
        Center,
        TopLeft,
        TopRight,
        Bottom,
        BottomLeft,
        BottomRight
    }

    impl ID {
        fn to_bits(&self) -> u32 {
            match *self {
                ID::Top         => 0b00,
                ID::Center      => 0b10,
                ID::TopLeft     => 0b01,
                ID::TopRight    => 0b11,
                ID::Bottom      => 0b00,
                ID::BottomLeft  => 0b01,
                ID::BottomRight => 0b11,
            }
        }

        fn from_bits(orientation: Orientation, bits: u32) -> ID {
            match orientation {
                Orientation::TipUp => match bits {
                    0b00 => ID::Top,
                    0b10 => ID::Center,
                    0b01 => ID::BottomLeft,
                    0b11 => ID::BottomRight,
                    _ => panic!("Not a valid CellID value."),
                },
                Orientation::TipDown => match bits {
                    0b00 => ID::Bottom,
                    0b10 => ID::Center,
                    0b01 => ID::TopLeft,
                    0b11 => ID::TopRight,
                    _ => panic!("Not a valid CellID value."),
                }
            }
        }
    }
}

pub struct LocationCode {
    level: u32,
    path: u32
}

pub struct CellData {
    location: LocationCode,
}

pub type CellIndex = usize;

pub enum Cell {
    Node {
        data: CellData,
        cells: [CellIndex; 4],
    },
    Leaf { data: CellData, }
}

pub const SUBDIVISION_LEVELS: usize = 5;
pub struct Grid {
    nodes: [Cell; (20 * 4) * SUBDIVISION_LEVELS],
}
