//! Grid lines pass

use amethyst::{
    core::{
        cgmath::{Matrix4, One},
        specs::{Join, Read, ReadStorage, Write, WriteStorage},
        transform::GlobalTransform,
    },
    renderer::{
        error::Result,
        get_camera,
        pipe::{
            pass::{Pass, PassData},
            DepthMode, Effect, NewEffect,
        },
        set_vertex_args, ActiveCamera, Camera, Color, Encoder, Factory, Mesh,
        Normal, Position, Query, Attributes,
    },
};
use origin_grid::{GridLine, GridLines, GridLinesComponent};
use gfx::pso::buffer::ElemStride;
use gfx::Primitive;
use glsl_layout::{Uniform, mat4};
use std::marker::PhantomData;


static VERT_SRC: &[u8] = include_bytes!("shaders/vertex/origin_grid.glsl");
static GEOM_SRC: &[u8] = include_bytes!("shaders/geometry/origin_grid.glsl");
static FRAG_SRC: &[u8] = include_bytes!("shaders/fragment/origin_grid.glsl");

fn set_attribute_buffers(
    effect: &mut Effect,
    mesh: &Mesh,
    attributes: &[Attributes<'static>],
) -> bool {
    for attr in attributes.iter() {
        match mesh.buffer(attr) {
            Some(vbuf) => effect.data.vertex_bufs.push(vbuf.clone()),
            None => {
                error!(
                    "Required vertex attribute buffer with format {:?} missing in mesh",
                    attr
                );
                return false;
            }
        }
    }
    true
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Uniform)]
pub(crate) struct VertexArgs {
    proj: mat4,
    view: mat4,
    model: mat4,
}

/// Parameters for renderer of debug lines. The params affect all lines.
pub struct GridLinesParams {
    /// Width of lines in units, default is 1.0 / 400.0 units
    pub line_width: f32,
}

impl Default for GridLinesParams {
    fn default() -> Self {
        GridLinesParams {
            line_width: 1.0 / 400.0,
        }
    }
}

/// Draw several simple lines for debugging
///
/// See the [crate level documentation](index.html) for information about interleaved and separate
/// passes.
///
/// # Type Parameters:
///
/// * `V`: `VertexFormat`
#[derive(Derivative, Clone, Debug, PartialEq)]
#[derivative(Default(bound = "V: Query<(Position, Color, Normal)>"))]
pub struct DrawGridLines<V> {
    _pd: PhantomData<V>,
}

impl<V> DrawGridLines<V>
    where
        V: Query<(Position, Color, Normal)>,
{
    /// Create instance of `DrawGridLines` pass
    pub fn new() -> Self {
        Default::default()
    }
}

impl<'a, V> PassData<'a> for DrawGridLines<V>
    where
        V: Query<(Position, Color, Normal)>,
{
    type Data = (
        Option<Read<'a, ActiveCamera>>,
        ReadStorage<'a, Camera>,
        ReadStorage<'a, GlobalTransform>,
        WriteStorage<'a, GridLinesComponent>, // GridLines components
        Option<Write<'a, GridLines>>,         // GridLines resource
        Read<'a, GridLinesParams>,
    );
}

impl<V> Pass for DrawGridLines<V>
    where
        V: Query<(Position, Color, Normal)>,
{
    fn compile(&mut self, effect: NewEffect) -> Result<Effect> {
        debug!("Building debug lines pass");
        let mut builder = effect.geom(VERT_SRC, GEOM_SRC, FRAG_SRC);

        debug!("Effect compiled, adding vertex/uniform buffers");
        builder.with_raw_vertex_buffer(V::QUERIED_ATTRIBUTES, V::size() as ElemStride, 0);

        builder.with_raw_constant_buffer(
            "VertexArgs",
            std::mem::size_of::<<VertexArgs as Uniform>::Std140>(),
            1
        );
        builder.with_raw_global("camera_position");
        builder.with_raw_global("line_width");
        builder.with_primitive_type(Primitive::PointList);
        builder.with_output("color", Some(DepthMode::LessEqualWrite));

        builder.build()
    }

    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoder: &mut Encoder,
        effect: &mut Effect,
        mut factory: Factory,
        (active, camera, global, lines_components, lines_resource, lines_params): <Self as PassData<'a>>::Data,
    ){
        trace!("Drawing debug lines pass");
        let debug_lines = {
            let mut lines = Vec::<GridLine>::new();

            for debug_lines_component in (&lines_components).join() {
                lines.extend(&debug_lines_component.lines);
            }

            if let Some(mut lines_resource) = lines_resource {
                lines.append(&mut lines_resource.lines);
            };

            lines
        };

        if debug_lines.len() == 0 {
            effect.clear();
            return;
        }

        let camera = get_camera(active, &camera, &global);
        effect.update_global(
            "camera_position",
            camera
                .as_ref()
                .map(|&(_, ref trans)| [trans.0[3][0], trans.0[3][1], trans.0[3][2]])
                .unwrap_or([0.0; 3]),
        );

        effect.update_global("line_width", lines_params.line_width);

        let mesh = Mesh::build(debug_lines)
            .build(&mut factory)
            .expect("Failed to create debug lines mesh");

        if !set_attribute_buffers(effect, &mesh, &[V::QUERIED_ATTRIBUTES]) {
            effect.clear();
            return;
        }

        set_vertex_args(effect, encoder, camera, &GlobalTransform(Matrix4::one()));

        effect.draw(mesh.slice(), encoder);
        effect.clear();
    }
}
