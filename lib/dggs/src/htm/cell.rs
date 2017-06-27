pub type Index = u32;
pub type Level = u32;

/// A path to a cell in the discrete global grid system
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Path(pub u32);

pub const ID_MASK: u32 = 0b00000000000000000000000000000011;
pub const EVEN_BITS: u32 = 0b01010101010101010101010101010101;
pub const ODD_BITS: u32 = 0b10101010101010101010101010101010;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum ChildType {
    Center = 0b10,
    Vert = 0b00,
    Left = 0b01,
    Right = 0b11,
}

impl Path {
    pub fn nearest_common_ancestor(self, direction: NeighborDirection, depth: usize) -> Path {
        let child_type = self.child_type();
        unimplemented!()
    }

    pub fn child_type(self) -> ChildType {
        match self.0 & ID_MASK {
            0b10 => ChildType::Center,
            0b00 => ChildType::Vert,
            0b01 => ChildType::Left,
            0b11 => ChildType::Right,
            _ => unreachable!(),
        }
    }

    pub fn parent(self) -> Path {
        Path(self.0 >> 2)
    }

    pub fn orientation(
        self,
        subdivision_level: Level,
        tree_orientation: Orientation,
    ) -> Orientation {
        let parent_orientation = match subdivision_level {
            1 => tree_orientation,
            _ => {
                self.parent().orientation(
                    subdivision_level - 1,
                    tree_orientation,
                )
            },
        };

        match self.child_type() {
            ChildType::Center => {
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
pub fn stop_tab(child_type: ChildType, direction: NeighborDirection) -> bool {
    match (child_type, direction) {
        (ChildType::Vert, NeighborDirection::Left) => false,
        (ChildType::Vert, NeighborDirection::Right) => false,
        (ChildType::Vert, NeighborDirection::Vert) => true,

        (ChildType::Left, NeighborDirection::Left) => false,
        (ChildType::Left, NeighborDirection::Right) => true,
        (ChildType::Left, NeighborDirection::Vert) => false,

        (ChildType::Center, NeighborDirection::Left) => true,
        (ChildType::Center, NeighborDirection::Right) => true,
        (ChildType::Center, NeighborDirection::Vert) => true,

        (ChildType::Right, NeighborDirection::Left) => true,
        (ChildType::Right, NeighborDirection::Right) => false,
        (ChildType::Right, NeighborDirection::Vert) => false,
    }
}
