use crate::{VoyagerError, VoyagerResult};

use log;
use std::cell::RefCell;

pub struct Pass<'enc> {
    frame: wgpu::SwapChainOutput<'enc>,
    pass: wgpu::RenderPass<'enc>,
}

pub struct Renderer {
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    encoder: RefCell<wgpu::CommandEncoder>,
}

impl Renderer {
    pub fn initialize(device: &mut wgpu::Device) -> VoyagerResult<Renderer> {
        let vs = include_bytes!("../../data/triangle.vert.spv");
        let fs = include_bytes!("../../data/triangle.frag.spv");

        log::trace!("Creating vertex shader module for default pipeline");
        let vs_module =
            device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&vs[..]))?);

        log::trace!("Creating fragment shader module for default pipeline");
        let fs_module =
            device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&fs[..]))?);

        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { bindings: &[] });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[],
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        let encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

        Ok(Renderer {
            bind_group,
            pipeline,
            encoder: RefCell::new(encoder),
        })
    }

    pub fn begin_pass<'a>(
        &self,
        encoder: &'a mut wgpu::CommandEncoder,
        swap_chain: &'a mut wgpu::SwapChain,
    ) -> Pass<'a> {
        let frame = swap_chain.get_next_texture();

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.view,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color::BLUE,
            }],
            depth_stencil_attachment: None,
        });
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        Pass {
            frame,
            pass: render_pass,
        }
    }
}
