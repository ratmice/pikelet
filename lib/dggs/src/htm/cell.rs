pub type Index = usize;
pub type Level = usize;
pub type Path = usize;

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

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Orientation {
    Up,
    Down,
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum NeighborDirection {
    Left,
    Right,
    Vert,
}

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

pub fn common_ancestor(excode: Path, direction: NeighborDirection, depth: usize) -> Path {
    let child_type = excode & ID_MASK;
    unimplemented!()
}

pub fn orientation_for_path(subdivision_level: Level,
                            tree_orientation: Orientation,
                            path: Path)
                            -> Orientation {
    let node_id = path & ID_MASK;
    let parent_orientation = if subdivision_level == 1 {
        tree_orientation
    } else {
        let parent_path = path >> 2;
        let parent_level = subdivision_level - 1;
        orientation_for_path(parent_level, tree_orientation, parent_path)
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
