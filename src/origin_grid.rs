use amethyst::{
    core::{
        cgmath::{Point3, Vector3},
        specs::{Component, DenseVecStorage},
    },
    renderer::{PosColorNorm, Rgba},
};

/// Debug lines are stored as a position, a direction and a color.
///
/// Storing a direction instead of an end position may not be intuitive,
/// but is similar to other 'VertexFormat's.
pub type GridLine = PosColorNorm;

/// Component that stores persistent debug lines to be rendered in GridLinesPass draw pass.
/// The vector can only be cleared manually.
#[derive(Debug, Default)]
pub struct GridLinesComponent {
    /// Lines to be rendered
    pub lines: Vec<GridLine>,
}

impl Component for GridLinesComponent {
    type Storage = DenseVecStorage<Self>;
}

impl GridLinesComponent {
    /// Creates a new debug lines component with an empty GridLine vector.
    pub fn new() -> GridLinesComponent {
        GridLinesComponent {
            lines: Vec::<GridLine>::new(),
        }
    }

    /// Builder method to pre-allocate a number of lines.
    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.lines = Vec::<GridLine>::with_capacity(capacity);
        self
    }

    /// Adds a line to be rendered by giving a position and a direction.
    pub fn add_direction(&mut self, position: Point3<f32>, direction: Vector3<f32>, color: Rgba) {
        let vertex = GridLine {
            position: position.into(),
            color: color.into(),
            normal: direction.into(),
        };

        self.lines.push(vertex);
    }

    /// Adds a line to be rendered by giving a start and an end position.
    pub fn add_line(&mut self, start: Point3<f32>, end: Point3<f32>, color: Rgba) {
        let vertex = GridLine {
            position: start.into(),
            color: color.into(),
            normal: (end - start).into(),
        };

        self.lines.push(vertex);
    }

    /// Clears lines buffer.
    ///
    /// As lines are persistent, it's necessary to use this function for updating or deleting lines.
    pub fn clear(&mut self) {
        self.lines.clear();
    }
}

/// Resource that stores non-persistent debug lines to be rendered in GridLinesPass draw pass.
/// The vector is automatically cleared after being rendered.
#[derive(Debug, Default)]
pub struct GridLines {
    /// Lines to be rendered
    pub lines: Vec<GridLine>,
}

impl GridLines {
    /// Creates a new debug lines component with an empty GridLine vector.
    pub fn new() -> GridLines {
        GridLines {
            lines: Vec::<GridLine>::new(),
        }
    }

    /// Builder method to pre-allocate a number of lines.
    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.lines = Vec::<GridLine>::with_capacity(capacity);
        self
    }

    /// Submits a line to be rendered by giving a position and a direction.
    pub fn draw_direction(&mut self, position: Point3<f32>, direction: Vector3<f32>, color: Rgba) {
        let vertex = GridLine {
            position: position.into(),
            color: color.into(),
            normal: direction.into(),
        };

        self.lines.push(vertex);
    }

    /// Submits a line to be rendered by giving a start and an end position.
    pub fn draw_line(&mut self, start: Point3<f32>, end: Point3<f32>, color: Rgba) {
        let vertex = GridLine {
            position: start.into(),
            color: color.into(),
            normal: (end - start).into(),
        };

        self.lines.push(vertex);
    }
}
