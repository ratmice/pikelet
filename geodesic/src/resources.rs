use glium::{IndexBuffer, Program, VertexBuffer};
use glium::backend::Context;
use glium::index::NoIndices;
use rusttype::Font;
use std::rc::Rc;

use text::Vertex as TextVertex;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
}

implement_vertex!(Vertex, position);

pub struct Resources {
    pub context: Rc<Context>,

    pub planet_vertex_buffer: Option<VertexBuffer<Vertex>>,
    pub index_buffer: NoIndices,

    pub stars0_vertex_buffer: VertexBuffer<Vertex>,
    pub stars1_vertex_buffer: VertexBuffer<Vertex>,
    pub stars2_vertex_buffer: VertexBuffer<Vertex>,

    pub text_vertex_buffer: VertexBuffer<TextVertex>,
    pub text_index_buffer: IndexBuffer<u8>,

    pub flat_shaded_program: Program,
    pub text_program: Program,
    pub unshaded_program: Program,

    pub blogger_sans_font: Font<'static>,
}
