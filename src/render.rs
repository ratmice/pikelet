use cgmath::conv::*;
use cgmath::{Matrix4, Point2, Vector3};
use glium::{self, index, program, texture, vertex};
use glium::{DrawParameters, Frame, PolygonMode, Surface};

use camera::ComputedCamera;
use color::Color;
use resources::Resources;
use text::TextData;

pub enum Command {
    Clear {
        color: Color,
    },
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

pub fn handle_command(frame: &mut Frame, resources: &Resources, command: Command) -> RenderResult<()> {
    let result = match command {
        Command::Clear { color } => {
            frame.clear_color_and_depth(color, 1.0);
            Some(Ok(()))
        },
        Command::Points { buffer_name, size, color, model, camera } => {
            let program = &resources.programs["unshaded"];
            let draw_params = DrawParameters { polygon_mode: PolygonMode::Point, point_size: Some(size), ..draw_params() };
            let uniforms = uniform! {
                color:      color,
                model:      array4x4(model),
                view:       array4x4(camera.view),
                proj:       array4x4(camera.projection),
            };

            resources.buffers.get(&buffer_name).map(|&(ref vbuf, ref ibuf)| {
                frame.draw(vbuf, ibuf, program, &uniforms, &draw_params)
            })
        },
        Command::Lines { buffer_name, width, color, model, camera } => {
            let program = &resources.programs["unshaded"];
            let draw_params = DrawParameters { polygon_mode: PolygonMode::Line, line_width: Some(width), ..draw_params() };
            let uniforms = uniform! {
                color:      color,
                model:      array4x4(model),
                view:       array4x4(camera.view),
                proj:       array4x4(camera.projection),
            };

            resources.buffers.get(&buffer_name).map(|&(ref vbuf, ref ibuf)| {
                frame.draw(vbuf, ibuf, program, &uniforms, &draw_params)
            })
        },
        Command::Solid { buffer_name, light_dir, color, model, camera } => {
            let program = &resources.programs["flat_shaded"];
            let draw_params = DrawParameters { polygon_mode: PolygonMode::Fill, ..draw_params() };
            let uniforms = uniform! {
                color:      color,
                light_dir:  array3(light_dir),
                model:      array4x4(model),
                view:       array4x4(camera.view),
                proj:       array4x4(camera.projection),
                eye:        array3(camera.position),
            };

            resources.buffers.get(&buffer_name).map(|&(ref vbuf, ref ibuf)| {
                frame.draw(vbuf, ibuf, program, &uniforms, &draw_params)
            })
        },
        Command::Text { font_name, color, text, size, position, screen_matrix } => {
            use glium::texture::Texture2d;
            use glium::uniforms::MagnifySamplerFilter;

            let font = match resources.fonts.get(&font_name) {
                Some(font) => font,
                None => return Ok(()),
            };
            let text_data = TextData::new(font, &text, size);
            let text_texture = try!(Texture2d::new(&resources.context, &text_data));

            Some(frame.draw(
                &resources.text_vertex_buffer,
                &resources.text_index_buffer,
                &resources.programs["text"],
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
