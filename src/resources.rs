use glium::{IndexBuffer, Program, VertexBuffer};
use glium::backend::{Context, Facade};
use glium::index::{PrimitiveType, NoIndices};
use rusttype::{Font, FontCollection};
use std::collections::HashMap;
use std::rc::Rc;

use text;
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
    UploadFont {
        name: String,
        data: Vec<u8>,
    },
}

pub type Buffer = (VertexBuffer<Vertex>, NoIndices);

pub struct Resources {
    pub context: Rc<Context>,

    pub buffers: HashMap<String, Buffer>,
    pub programs: HashMap<String, Program>,
    pub fonts: HashMap<String, Font<'static>>,

    pub text_vertex_buffer: VertexBuffer<TextVertex>,
    pub text_index_buffer: IndexBuffer<u8>,
}

impl Resources {
    pub fn new<F: Facade>(facade: &F) -> Resources {
        Resources {
            context: facade.get_context().clone(),

            buffers: HashMap::new(),
            programs: HashMap::new(),
            fonts: HashMap::new(),

            text_vertex_buffer: VertexBuffer::new(facade, &text::TEXTURE_VERTICES).unwrap(),
            text_index_buffer: IndexBuffer::new(facade, PrimitiveType::TrianglesList, &text::TEXTURE_INDICES).unwrap(),
        }
    }

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
            Event::UploadFont { name, data } => {
                let font_collection = FontCollection::from_bytes(data);
                let font = font_collection.into_font().unwrap();

                self.fonts.insert(name, font);
            },
        }
    }
}
