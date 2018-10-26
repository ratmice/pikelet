//! The implementation of the Discrete Global Grid System (DGGS)
//!
//! # Designing a Discrete Global Grid System
//!
//! In order to move forward, we must experiment with the following design choices [1]:
//!
//! 1. A base regular polyhedron.
//! 2. A fixed orientation of the base regular polyhedron relative to the planet
//! 3. A hierarchical spatial partitioning method defined symmetrically on a face (or set of faces)
//!    of the base regular polyhedron
//! 4. A method for transforming that planar partition to the corresponding spherical surface
//! 5. A method for assigning points to grid cells
//!
//! ## Base Polyhedron
//!
//! The the five platonic solids offer the only way to partition the sphere into cells where:
//!
//! - the cells consist of same regular polygon
//! - the number of polygons meeting at each vertex are the same
//!
//! Platonic solids with a lower number of vertices reduce the amount of distortion when
//! transforming between the face and the surface of the sphere. The icosahedron has the highest
//! number of faces, and so has the least amount of distortion.
//!
//! | Platonic Solid    | Vertices | Edges | Faces |
//! | ----------------- | -------- | ----- | ----- |
//! | tetrahedron       |        4 |     6 |     4 |
//! | hexahedron (cube) |        8 |    12 |     6 |
//! | octahedron        |        6 |    12 |     8 |
//! | dodecahedron      |       20 |    30 |    12 |
//! | icosahedron       |       12 |    30 |    20 |
//!
//! ## Orientation
//!
//! This is more of a concern for those modelling the actual earth.
//!
//! ## Spatial partitioning method
//!
//! ```text
//!
//!          +---+
//!         /     \
//!    +---+  ,x,  +---+
//!   /    ,'     ',    \
//!  +   x'  +---+  'x   +
//!   \  |  /     \  |  /
//!    +-|-+   @   +-|-+
//!   /  |  \     /  |  \
//!  +   x,  +---+  ,x   +
//!   \    ',     ,'     /
//!    +---+  'x'   +---+
//!         \     /
//!          +---+
//!
//! ```
//!
//! ```text
//!                 ,+,
//!              +'     '+
//!              |   @   |
//!             ,+,     ,+,
//!          +'     '+'     '+
//!          |   x---|---x   |
//!         ,+ ,'   ,+,   ', +,
//!      +'   ,' +'     '+ ',   '+
//!      |   x   |   @   |   x   |
//!     ,+,   ', +,     ,+ ,'   ,+,
//!  +'     '+ ',   '+'   , '+'     '+
//!  |   @   |   x---|---x   |   @   |
//!  +,     ,+,     ,+,     ,+,     ,+
//!     '+'     '+'     '+'     '+'
//!
//! ```
//!
//! ## Transformation method
//!
//! ## Grid cell assignment method
//!
//!
//!
//! # Generalized Balanced Ternary (GBT)
//!
//! ```text
//!
//!             ,+,     ,+,
//!          +'  5₈ '+'  1₈ '+
//!          |  101₂ |  001₂ |
//!         ,+,     ,+,     ,+,
//!      +'  4₈ '+'  0₈ '+'  3₈ '+
//!      |  100₂ |  000₂ |  011₂ |
//!      +,     ,+,     ,+,     ,+
//!         '+'  6₈ '+'  2₈ '+'
//!          |  110₂ |  101₂ |
//!          +,     ,+,     ,+
//!             '+'     '+'
//!
//! ```
//!
//! 3 bits per location. Remaining bit can be used to indicate termination.
//!
//! ```text
//!
//! XXX1 XXX1 XXX1 XXX1 XXX0 XXX1 XXX1 XXX0
//! |______________________| |____________|
//!          Loc A               Loc B
//!
//! ```
//!
//! [1] K. Sahr, D. White, and A. J. Kimerling, “Geodesic discrete global grid systems,”
//!     _Cartography and Geographic Information Systems_, vol. 30, no. 2, pp. 121–134, 2003.
//!
#![allow(dead_code, unused_variables)]

pub mod htm;

pub struct Layer {
    level: usize,
}

pub struct Dggs {
    layers: Vec<Layer>,
}

impl Dggs {
    pub fn new(levels: usize) -> Dggs {
        let layers: Vec<Layer> = Vec::with_capacity(levels);
        unimplemented!()
    }

    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }
}
