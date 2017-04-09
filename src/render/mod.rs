extern crate rusttype;

use cgmath::conv::*;
use cgmath::{Matrix4, Point2, Vector3};
use glium::{self, index, program, texture, vertex};
use glium::{DrawParameters, Frame, IndexBuffer, PolygonMode, Program, Surface, VertexBuffer};
use glium::backend::{Context, Facade};
use glium::index::{PrimitiveType, NoIndices};
use self::rusttype::{Font, FontCollection};
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use camera::ComputedCamera;
use color::Color;
use self::text::{TextData, TextVertex};

mod text;

enum DrawCommand {
    Clear { color: Color },
    Points {
        buffer_name: String,
        size: f32,
        color: Color,
        model: Matrix4<f32>,
        camera: ComputedCamera,
    },
    Lines {
        buffer_name: String,
        width: f32,
        color: Color,
        model: Matrix4<f32>,
        camera: ComputedCamera,
    },
    Solid {
        buffer_name: String,
        light_dir: Vector3<f32>,
        color: Color,
        model: Matrix4<f32>,
        camera: ComputedCamera,
    },
    Text {
        font_name: String,
        color: Color,
        text: String,
        size: f32,
        position: Point2<f32>,
        screen_matrix: Matrix4<f32>,
    },
}

pub struct CommandList {
    commands: Vec<DrawCommand>,
}

impl CommandList {
    pub fn new() -> CommandList {
        CommandList { commands: Vec::new() }
    }

    pub fn clear(&mut self, color: Color) {
        self.commands.push(DrawCommand::Clear { color: color });
    }

    pub fn points<S>(&mut self,
                     buffer_name: S,
                     size: f32,
                     color: Color,
                     model: Matrix4<f32>,
                     camera: ComputedCamera)
        where S: Into<String>
    {
        self.commands
            .push(DrawCommand::Points {
                      buffer_name: buffer_name.into(),
                      size: size,
                      color: color,
                      model: model,
                      camera: camera,
                  });
    }

    pub fn lines<S>(&mut self,
                    buffer_name: S,
                    width: f32,
                    color: Color,
                    model: Matrix4<f32>,
                    camera: ComputedCamera)
        where S: Into<String>
    {
        self.commands
            .push(DrawCommand::Lines {
                      buffer_name: buffer_name.into(),
                      width: width,
                      color: color,
                      model: model,
                      camera: camera,
                  });
    }

    pub fn solid<S>(&mut self,
                    buffer_name: S,
                    light_dir: Vector3<f32>,
                    color: Color,
                    model: Matrix4<f32>,
                    camera: ComputedCamera)
        where S: Into<String>
    {
        self.commands
            .push(DrawCommand::Solid {
                      buffer_name: buffer_name.into(),
                      light_dir: light_dir,
                      color: color,
                      model: model,
                      camera: camera,
                  });
    }

    pub fn text<S>(&mut self,
                   font_name: S,
                   color: Color,
                   text: String,
                   size: f32,
                   position: Point2<f32>,
                   screen_matrix: Matrix4<f32>)
        where S: Into<String>
    {
        self.commands
            .push(DrawCommand::Text {
                      font_name: font_name.into(),
                      color: color,
                      text: text,
                      size: size,
                      position: position,
                      screen_matrix: screen_matrix,
                  });
    }
}

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

    buffers: HashMap<String, Buffer>,
    programs: HashMap<String, Program>,
    fonts: HashMap<String, Font<'static>>,

    text_vertex_buffer: VertexBuffer<TextVertex>,
    text_index_buffer: IndexBuffer<u8>,
}

impl Resources {
    pub fn new<F: Facade>(facade: &F) -> Resources {
        Resources {
            context: facade.get_context().clone(),

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

    pub fn handle_event(&mut self, event: ResourceEvent) {
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

    fn handle_draw_command(&self, frame: &mut Frame, command: DrawCommand) -> RenderResult<()> {
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
        };

        match result {
            Some(Ok(())) | None => Ok(()),
            Some(Err(err)) => Err(RenderError::from(err)),
        }
    }

    pub fn draw(&self, frame: &mut Frame, command_list: CommandList) -> RenderResult<()> {
        for command in command_list.commands {
            self.handle_draw_command(frame, command)?;
        }

        Ok(())
    }
}
