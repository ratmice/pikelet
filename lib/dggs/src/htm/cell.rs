pub type Index = usize;
pub type Level = usize;

/// A path to a cell in the discrete global grid system
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Path(pub usize);

pub const ID_MASK: usize = 0x03;
pub const EVEN_BITS: usize = 0xAAAAAAAAAAAAAAAA;
pub const ODD_BITS: usize = 0x5555555555555555;

pub const CENTER: usize = 0b10;

pub const TOP: usize = 0b00;
pub const TOP_LEFT: usize = 0b01;
pub const TOP_RIGHT: usize = 0b11;

pub const BOTTOM: usize = 0b00;
pub const BOTTOM_LEFT: usize = 0b01;
pub const BOTTOM_RIGHT: usize = 0b11;

impl Path {
    pub fn nearest_common_ancestor(self, direction: NeighborDirection, depth: usize) -> Path {
        let child_type = self.0 & ID_MASK;
        unimplemented!()
    }

    pub fn parent(self) -> Path {
        Path(self.0 >> 2)
    }

    pub fn orientation(self,
                       subdivision_level: Level,
                       tree_orientation: Orientation)
                       -> Orientation {
        let node_id = self.0 & ID_MASK;
        let parent_orientation = match subdivision_level {
            1 => tree_orientation,
            _ => self.parent().orientation(subdivision_level - 1, tree_orientation),
        };

        match node_id {
            CENTER => {
                match parent_orientation {
                    Orientation::Up => Orientation::Down,
                    Orientation::Down => Orientation::Up,
                }
            },
            _ => parent_orientation,
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Orientation {
    /// ```text
    ///         +
    ///        / \
    ///       /   \
    ///      / 00  \
    ///     +-------+
    ///    / \ 10  / \
    ///   /   \   /   \
    ///  / 01  \ / 11  \
    /// /-------+-------+
    /// ```
    Up,
    /// ```text
    /// +-------+-------+
    ///  \ 01  / \ 11  /
    ///   \   /   \   /
    ///    \ / 10  \ /
    ///     +-------+
    ///      \ 00  /
    ///       \   /
    ///        \ /
    ///         +
    /// ```
    Down,
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum NeighborDirection {
    Left,
    Right,
    Vert,
}

/// Indicates when to cease search for the `nearest_common_ancestor`
pub fn stop_tab(child_type: u8, direction: NeighborDirection) -> bool {
    match (child_type, direction) {
        (0b00, NeighborDirection::Left) => false,
        (0b00, NeighborDirection::Right) => false,
        (0b00, NeighborDirection::Vert) => true,

        (0b01, NeighborDirection::Left) => false,
        (0b01, NeighborDirection::Right) => true,
        (0b01, NeighborDirection::Vert) => false,

        (0b10, NeighborDirection::Left) => true,
        (0b10, NeighborDirection::Right) => true,
        (0b10, NeighborDirection::Vert) => true,

        (0b11, NeighborDirection::Left) => true,
        (0b11, NeighborDirection::Right) => false,
        (0b11, NeighborDirection::Vert) => false,

        _ => panic!("unreachable!"),
    }
}
