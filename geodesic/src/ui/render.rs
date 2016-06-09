use cgmath;
use cgmath::conv::*;
use glium::{IndexBuffer, Program, Surface, Texture2d, VertexBuffer};
use glium::backend::{Context, Facade};
use glium::index::PrimitiveType;
use glium::program::ProgramChooserCreationError;
use imgui::{DrawList, Ui, ImDrawIdx, ImDrawVert, ImGui};
use std::borrow::Cow;
use std::rc::Rc;

use render::RenderResult;

pub struct Renderer {
    ctx: Rc<Context>,
    device_objects: DeviceObjects
}

impl Renderer {
    pub fn init<F: Facade>(imgui: &mut ImGui, ctx: &F) -> RenderResult<Renderer> {
        let device_objects = try!(DeviceObjects::init(imgui, ctx));
        Ok(Renderer {
            ctx: ctx.get_context().clone(),
            device_objects: device_objects
        })
    }

    pub fn render<'a, S: Surface>(&mut self, surface: &mut S, ui: Ui<'a>, hidpi_factor: f32) -> RenderResult<()> {
        ui.render(|draw_list| self.render_draw_list(surface, draw_list, hidpi_factor))
    }

    fn render_draw_list<'a, S: Surface>(&mut self, surface: &mut S, draw_list: DrawList<'a>, hidpi_factor: f32) -> RenderResult<()> {
        use glium::{Blend, DrawParameters, Rect};
        use glium::uniforms::MagnifySamplerFilter;

        try!(self.device_objects.upload_vertex_buffer(&self.ctx, draw_list.vtx_buffer));
        try!(self.device_objects.upload_index_buffer(&self.ctx, draw_list.idx_buffer));

        let (width, height) = surface.get_dimensions();

        let mut idx_start = 0;

        for cmd in draw_list.cmd_buffer {
            let matrix = cgmath::ortho(
                0.0, width as f32 / hidpi_factor,
                height as f32 / hidpi_factor, 0.0,
                -1.0, 1.0,
            );

            let idx_end = idx_start + cmd.elem_count as usize;
            try!(surface.draw(
                &self.device_objects.vertex_buffer,
                &self.device_objects.index_buffer.slice(idx_start ..idx_end)
                    .expect("Invalid index buffer range"),
                &self.device_objects.program,
                &uniform! {
                    matrix: array4x4(matrix),
                    tex: self.device_objects.texture.sampled()
                        .magnify_filter(MagnifySamplerFilter::Nearest),
                },
                &DrawParameters {
                    blend: Blend::alpha_blending(),
                    scissor: Some(Rect {
                        left: (cmd.clip_rect.x * hidpi_factor) as u32,
                        bottom: (height as f32 - (cmd.clip_rect.w * hidpi_factor)) as u32,
                        width: ((cmd.clip_rect.z - cmd.clip_rect.x) * hidpi_factor) as u32,
                        height: ((cmd.clip_rect.w - cmd.clip_rect.y) * hidpi_factor) as u32,
                    }),
                    ..DrawParameters::default()
                },
            ));
            idx_start = idx_end;
        }
        Ok(())
    }
}

pub struct DeviceObjects {
    vertex_buffer: VertexBuffer<ImDrawVert>,
    index_buffer: IndexBuffer<ImDrawIdx>,
    program: Program,
    texture: Texture2d,
}

#[cfg_attr(feature = "clippy", allow(redundant_closure))]
fn compile_default_program<F: Facade>(ctx: &F) -> Result<Program, ProgramChooserCreationError> {
    program!(
        ctx,
        140 => {
            vertex: include_str!("shader/vert_140.glsl"),
            fragment: include_str!("shader/frag_140.glsl"),
            outputs_srgb: true,
        },
        110 => {
            vertex: include_str!("shader/vert_110.glsl"),
            fragment: include_str!("shader/frag_110.glsl"),
            outputs_srgb: true,
        }
    )
}

impl DeviceObjects {
    pub fn init<F: Facade>(im_gui: &mut ImGui, ctx: &F) -> RenderResult<DeviceObjects> {
        use glium::texture::{ClientFormat, RawImage2d};

        let vertex_buffer = try!(VertexBuffer::empty_dynamic(ctx, 0));
        let index_buffer = try!(IndexBuffer::empty_dynamic(ctx, PrimitiveType::TrianglesList, 0));

        let program = try!(compile_default_program(ctx));
        let texture = try!(im_gui.prepare_texture(|handle| {
            let data = RawImage2d {
                data: Cow::Borrowed(handle.pixels),
                width: handle.width,
                height: handle.height,
                format: ClientFormat::U8U8U8U8,
            };
            Texture2d::new(ctx, data)
        }));

        Ok(DeviceObjects {
            vertex_buffer: vertex_buffer,
            index_buffer: index_buffer,
            program: program,
            texture: texture,
        })
    }

    pub fn upload_vertex_buffer<F: Facade>(&mut self, ctx: &F,
                                           vtx_buffer: &[ImDrawVert]) -> RenderResult<()> {
        self.vertex_buffer.invalidate();
        if let Some(slice) = self.vertex_buffer.slice_mut(0..vtx_buffer.len()) {
            slice.write(vtx_buffer);
            return Ok(());
        }
        self.vertex_buffer = try!(VertexBuffer::dynamic(ctx, vtx_buffer));
        Ok(())
    }

    pub fn upload_index_buffer<F: Facade>(&mut self, ctx: &F,
                                          idx_buffer: &[ImDrawIdx]) -> RenderResult<()> {
        self.index_buffer.invalidate();
        if let Some(slice) = self.index_buffer.slice_mut(0..idx_buffer.len()) {
            slice.write(idx_buffer);
            return Ok(());
        }
        self.index_buffer = try!(IndexBuffer::dynamic(ctx, PrimitiveType::TrianglesList, idx_buffer));
        Ok(())
    }
}
