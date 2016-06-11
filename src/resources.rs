use glium::{IndexBuffer, Program, VertexBuffer};
use glium::backend::Context;
use glium::index::NoIndices;
use rusttype::Font;
use std::collections::HashMap;
use std::rc::Rc;

use text::Vertex as TextVertex;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
}

implement_vertex!(Vertex, position);

pub type Buffer = (VertexBuffer<Vertex>, NoIndices);

pub struct Resources {
    pub context: Rc<Context>,

    pub buffers: HashMap<String, Buffer>,
    pub programs: HashMap<String, Program>,

    pub text_vertex_buffer: VertexBuffer<TextVertex>,
    pub text_index_buffer: IndexBuffer<u8>,
    pub blogger_sans_font: Font<'static>,
}
