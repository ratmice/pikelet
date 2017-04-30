extern crate rusttype;

use cgmath::conv::*;
use glium::{self, glutin, index, program, texture, vertex};
use glium::{DrawParameters, Frame, IndexBuffer, PolygonMode, Program, Surface, VertexBuffer};
use glium::backend::{Context, Facade};
use glium::index::{PrimitiveType, NoIndices};
use imgui::ImGui;
use imgui::glium_renderer::{Renderer as UiRenderer, RendererError as UiRendererError};
use self::rusttype::{Font, FontCollection};
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use FrameMetrics;
use self::command::DrawCommand;
use self::text::{TextData, TextVertex};
use ui::Context as UiContext;

pub use self::command::CommandList;

mod text;
mod command;

pub type RenderResult<T> = Result<T, RenderError>;

quick_error! {
    #[derive(Debug)]
    pub enum RenderError {
        Draw(error: glium::DrawError) {
            from()
            description(error.description())
            cause(error)
        }
        Index(error: index::BufferCreationError) {
            from()
            description(error.description())
            cause(error)
        }
        Program(error: program::ProgramChooserCreationError) {
            from()
            description(error.description())
            cause(error)
        }
        Texture(error: texture::TextureCreationError) {
            from()
            description(error.description())
            cause(error)
        }
        Vertex(error: vertex::BufferCreationError) {
            from()
            description(error.description())
            cause(error)
        }
        Ui(error: UiRendererError) {
            from()
        }
    }
}

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
}

implement_vertex!(Vertex, position);

#[derive(Copy, Clone, Debug)]
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

pub enum ResourceEvent {
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
    UploadFont { name: String, data: Vec<u8> },
}

impl fmt::Debug for ResourceEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ResourceEvent::UploadBuffer {
                ref name,
                ref vertices,
                ref indices,
            } => {
                write!(f,
                       "ResourceEvent::UploadBuffer {{ name: {:?}, vertices: vec![_; {}], indices: {:?} }}",
                       name,
                       vertices.len(),
                       indices)
            },
            ResourceEvent::CompileProgram { ref name, .. } => {
                write!(f,
                       "ResourceEvent::CompileProgram {{ name: {:?}, vertex_shader: \"..\", fragment_shader: \"..\"] }}",
                       name)
            },
            ResourceEvent::UploadFont { ref name, ref data } => {
                write!(f,
                       "ResourceEvent::UploadFont {{ name: {:?}, data: vec![_; {}] }}",
                       name,
                       data.len())
            },
        }
    }
}

pub type Buffer = (VertexBuffer<Vertex>, NoIndices);

pub struct Resources {
    context: Rc<Context>,

    ui_renderer: UiRenderer,
    ui_context: UiContext,
    ui_was_rendered: bool,

    buffers: HashMap<String, Buffer>,
    programs: HashMap<String, Program>,
    fonts: HashMap<String, Font<'static>>,

    text_vertex_buffer: VertexBuffer<TextVertex>,
    text_index_buffer: IndexBuffer<u8>,
}

impl Resources {
    pub fn new<F: Facade>(facade: &F) -> Resources {
        let mut imgui = ImGui::init();
        let ui_renderer = UiRenderer::init(&mut imgui, facade).unwrap();
        let ui_context = UiContext::new(imgui);

        Resources {
            context: facade.get_context().clone(),

            ui_renderer: ui_renderer,
            ui_context: ui_context,
            ui_was_rendered: false,

            buffers: HashMap::new(),
            programs: HashMap::new(),
            fonts: HashMap::new(),

            text_vertex_buffer: VertexBuffer::new(facade, &text::TEXTURE_VERTICES).unwrap(),
            text_index_buffer: IndexBuffer::new(facade,
                                                PrimitiveType::TrianglesList,
                                                &text::TEXTURE_INDICES)
                    .unwrap(),
        }
    }

    pub fn handle_ui_event(&mut self, event: glutin::Event) {
        if self.ui_was_rendered {
            self.ui_context.update(event);
        }
    }

    pub fn handle_resource_event(&mut self, event: ResourceEvent) {
        match event {
            ResourceEvent::UploadBuffer {
                name,
                vertices,
                indices,
            } => {
                let vbo = VertexBuffer::new(&self.context, &vertices).unwrap();
                let ibo = indices.to_no_indices();

                self.buffers.insert(name, (vbo, ibo));
            },
            ResourceEvent::CompileProgram {
                name,
                vertex_shader,
                fragment_shader,
            } => {
                let program =
                    Program::from_source(&self.context, &vertex_shader, &fragment_shader, None)
                        .unwrap();

                self.programs.insert(name, program);
            },
            ResourceEvent::UploadFont { name, data } => {
                let font_collection = FontCollection::from_bytes(data);
                let font = font_collection.into_font().unwrap();

                self.fonts.insert(name, font);
            },
        }
    }

    fn handle_draw_command<Event, F>(&mut self,
                                     frame: &mut Frame,
                                     frame_metrics: FrameMetrics,
                                     command: DrawCommand<Event>,
                                     on_event: &mut F)
                                     -> RenderResult<()>
        where F: FnMut(Event)
    {
        fn draw_params<'a>() -> DrawParameters<'a> {
            use glium::{BackfaceCullingMode, Depth, DepthTest};

            DrawParameters {
                backface_culling: BackfaceCullingMode::CullClockwise,
                depth: Depth {
                    test: DepthTest::IfLess,
                    write: true,
                    ..Depth::default()
                },
                ..DrawParameters::default()
            }
        }

        let result = match command {
            DrawCommand::Clear { color } => {
                frame.clear_color_and_depth(color, 1.0);
                Some(Ok(()))
            },
            DrawCommand::Points {
                buffer_name,
                size,
                color,
                model,
                camera,
            } => {
                let program = &self.programs["unshaded"];
                let draw_params = DrawParameters {
                    polygon_mode: PolygonMode::Point,
                    point_size: Some(size),
                    ..draw_params()
                };
                let uniforms = uniform! {
                    color:      color,
                    model:      array4x4(model),
                    view:       array4x4(camera.view),
                    proj:       array4x4(camera.projection),
                };

                self.buffers
                    .get(&buffer_name)
                    .map(|&(ref vbuf, ref ibuf)| {
                             frame.draw(vbuf, ibuf, program, &uniforms, &draw_params)
                         })
            },
            DrawCommand::Lines {
                buffer_name,
                width,
                color,
                model,
                camera,
            } => {
                let program = &self.programs["unshaded"];
                let draw_params = DrawParameters {
                    polygon_mode: PolygonMode::Line,
                    line_width: Some(width),
                    ..draw_params()
                };
                let uniforms = uniform! {
                    color:      color,
                    model:      array4x4(model),
                    view:       array4x4(camera.view),
                    proj:       array4x4(camera.projection),
                };

                self.buffers
                    .get(&buffer_name)
                    .map(|&(ref vbuf, ref ibuf)| {
                             frame.draw(vbuf, ibuf, program, &uniforms, &draw_params)
                         })
            },
            DrawCommand::Solid {
                buffer_name,
                light_dir,
                color,
                model,
                camera,
            } => {
                let program = &self.programs["flat_shaded"];
                let draw_params = DrawParameters {
                    polygon_mode: PolygonMode::Fill,
                    ..draw_params()
                };
                let uniforms = uniform! {
                    color:      color,
                    light_dir:  array3(light_dir),
                    model:      array4x4(model),
                    view:       array4x4(camera.view),
                    proj:       array4x4(camera.projection),
                    eye:        array3(camera.position),
                };

                self.buffers
                    .get(&buffer_name)
                    .map(|&(ref vbuf, ref ibuf)| {
                             frame.draw(vbuf, ibuf, program, &uniforms, &draw_params)
                         })
            },
            DrawCommand::Text {
                font_name,
                color,
                text,
                size,
                position,
                screen_matrix,
            } => {
                use glium::texture::Texture2d;
                use glium::uniforms::MagnifySamplerFilter;

                let font = match self.fonts.get(&font_name) {
                    Some(font) => font,
                    None => return Ok(()),
                };
                let text_data = TextData::new(font, &text, size);
                let text_texture = Texture2d::new(&self.context, &text_data)?;

                Some(frame.draw(
                    &self.text_vertex_buffer,
                    &self.text_index_buffer,
                    &self.programs["text"],
                    &uniform! {
                        color:    color,
                        text:     text_texture.sampled().magnify_filter(MagnifySamplerFilter::Nearest),
                        proj:     array4x4(screen_matrix),
                        model:    array4x4(text_data.matrix(position)),
                    },
                    &{
                        use glium::Blend;
                        use glium::BlendingFunction::Addition;
                        use glium::LinearBlendingFactor::*;

                        let blending_function = Addition {
                            source: SourceAlpha,
                            destination: OneMinusSourceAlpha
                        };

                        DrawParameters {
                            blend: Blend {
                                color: blending_function,
                                alpha: blending_function,
                                constant_value: (1.0, 1.0, 1.0, 1.0),
                            },
                            ..DrawParameters::default()
                        }
                    },
                ))
            },
            DrawCommand::Ui { run_ui } => {
                self.ui_was_rendered = true;
                let ui = self.ui_context.frame(frame_metrics);

                for event in run_ui(&ui) {
                    on_event(event);
                }

                self.ui_renderer.render(frame, ui)?;

                Some(Ok(()))
            },
        };

        match result {
            Some(Ok(())) | None => Ok(()),
            Some(Err(err)) => Err(RenderError::from(err)),
        }
    }

    pub fn draw<Event, F>(&mut self,
                          frame: &mut Frame,
                          frame_metrics: FrameMetrics,
                          command_list: CommandList<Event>,
                          mut on_event: F)
                          -> RenderResult<()>
        where F: FnMut(Event)
    {
        self.ui_was_rendered = false;

        for command in command_list {
            self.handle_draw_command(frame, frame_metrics, command, &mut on_event)?;
        }

        Ok(())
    }
}
