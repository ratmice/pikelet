use glium::{IndexBuffer, Program, VertexBuffer};
use glium::backend::Context;
use glium::index::{PrimitiveType, NoIndices};
use rusttype::Font;
use std::collections::HashMap;
use std::rc::Rc;

use text::Vertex as TextVertex;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
}

implement_vertex!(Vertex, position);

#[derive(Copy, Clone)]
pub enum Indices {
    TrianglesList,
    Points,
}

impl Indices {
    fn to_no_indices(&self) -> NoIndices {
        match *self {
            Indices::TrianglesList => NoIndices(PrimitiveType::TrianglesList),
            Indices::Points => NoIndices(PrimitiveType::Points),
        }
    }
}

pub enum Event {
    UploadBuffer {
        name: String,
        vertices: Vec<Vertex>,
        indices: Indices,
    },
    CompileProgram {
        name: String,
        vertex_shader: String,
        fragment_shader: String,
    },
}

pub type Buffer = (VertexBuffer<Vertex>, NoIndices);

pub struct Resources {
    pub context: Rc<Context>,

    pub buffers: HashMap<String, Buffer>,
    pub programs: HashMap<String, Program>,

    pub text_vertex_buffer: VertexBuffer<TextVertex>,
    pub text_index_buffer: IndexBuffer<u8>,
    pub blogger_sans_font: Font<'static>,
}

impl Resources {
    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::UploadBuffer { name, vertices, indices } => {
                let vbo = VertexBuffer::new(&self.context, &vertices).unwrap();
                let ibo = indices.to_no_indices();

                self.buffers.insert(name, (vbo, ibo));
            },
            Event::CompileProgram { name, vertex_shader, fragment_shader } => {
                let program = Program::from_source(&self.context, &vertex_shader, &fragment_shader, None).unwrap();

                self.programs.insert(name, program);
            },
        }
    }
}
