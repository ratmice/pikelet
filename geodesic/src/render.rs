use cgmath::Angle;
use cgmath::{Matrix4, SquareMatrix};
use cgmath::Point2;
use cgmath::Vector3;
use glium::{Frame, IndexBuffer, Program, VertexBuffer};
use glium::{DrawParameters, PolygonMode, Surface};
use glium::backend::Context;
use glium::index::NoIndices;
use rusttype::Font;
use std::rc::Rc;

use camera::ComputedCamera;
use color::Color;
use math;
use text::TextData;
use text::Vertex as TextVertex;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
}

implement_vertex!(Vertex, position);

fn draw_params<'a>() -> DrawParameters<'a> {
    use glium::{BackfaceCullingMode, Depth, DepthTest};
    use glium::draw_parameters::{Smooth};

    DrawParameters {
        backface_culling: BackfaceCullingMode::CullClockwise,
        depth: Depth {
            test: DepthTest::IfLess,
            write: true,
            ..Depth::default()
        },
        smooth: Some(Smooth::Nicest),
        ..DrawParameters::default()
    }
}

pub struct Resources {
    pub context: Rc<Context>,

    pub delaunay_vertex_buffer: VertexBuffer<Vertex>,
    pub voronoi_vertex_buffer: VertexBuffer<Vertex>,
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

pub struct RenderTarget<'a> {
    pub frame: &'a mut Frame,
    pub hidpi_factor: f32,
    pub resources: &'a Resources,
    pub camera: ComputedCamera,
    pub hud_matrix: Matrix4<f32>,
}

impl<'a> RenderTarget<'a> {
    pub fn clear(&mut self, color: Color) {
        self.frame.clear_color_and_depth(color, 1.0);
    }

    pub fn render_hud_text(&mut self, text: &str, text_size: f32, position: Point2<f32>, color: Color) {
        use glium::texture::Texture2d;
        use glium::uniforms::MagnifySamplerFilter;

        let text_data = TextData::new(&self.resources.blogger_sans_font, text, text_size * self.hidpi_factor);
        let text_texture = Texture2d::new(&self.resources.context, &text_data).unwrap();

        let params = {
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
        };

        self.frame.draw(
            &self.resources.text_vertex_buffer,
            &self.resources.text_index_buffer,
            &self.resources.text_program,
            &uniform! {
                color:    color,
                text:     text_texture.sampled().magnify_filter(MagnifySamplerFilter::Nearest),
                proj:     math::array_m4(self.hud_matrix),
                model:    math::array_m4(text_data.matrix(position * self.hidpi_factor)),
            },
            &params,
        ).unwrap();
    }

    pub fn render_points(&mut self, vertex_buffer: &VertexBuffer<Vertex>, point_size: f32, color: Color) {
        self.frame.draw(
            vertex_buffer,
            &self.resources.index_buffer,
            &self.resources.unshaded_program,
            &uniform! {
                color:      color,
                model:      math::array_m4(Matrix4::from_scale(1.025)),
                view:       math::array_m4(self.camera.view),
                proj:       math::array_m4(self.camera.projection),
            },
            &DrawParameters {
                polygon_mode: PolygonMode::Point,
                point_size: Some(point_size),
                ..draw_params()
            },
        ).unwrap();
    }

    pub fn render_lines(&mut self, vertex_buffer: &VertexBuffer<Vertex>, line_width: f32, color: Color) {
        self.frame.draw(
            vertex_buffer,
            &self.resources.index_buffer,
            &self.resources.unshaded_program,
            &uniform! {
                color:      color,
                model:      math::array_m4(Matrix4::from_scale(1.025)),
                view:       math::array_m4(self.camera.view),
                proj:       math::array_m4(self.camera.projection),
            },
            &DrawParameters {
                polygon_mode: PolygonMode::Line,
                line_width: Some(line_width),
                ..draw_params()
            },
        ).unwrap();
    }

    pub fn render_solid(&mut self, vertex_buffer: &VertexBuffer<Vertex>, light_dir: Vector3<f32>, color: Color) {
        self.frame.draw(
            vertex_buffer,
            &self.resources.index_buffer,
            &self.resources.flat_shaded_program,
            &uniform! {
                color:      color,
                light_dir:  math::array_v3(light_dir),
                model:      math::array_m4(Matrix4::identity()),
                view:       math::array_m4(self.camera.view),
                proj:       math::array_m4(self.camera.projection),
                eye:        math::array_p3(self.camera.position),
            },
            &DrawParameters {
                polygon_mode: PolygonMode::Fill,
                ..draw_params()
            },
        ).unwrap();
    }
}
