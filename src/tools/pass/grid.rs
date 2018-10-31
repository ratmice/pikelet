//! Grid lines pass

use amethyst::{
    core::{
        cgmath::{Matrix4, One, Point3, Vector3},
        specs::{Read, ReadStorage},
        transform::GlobalTransform,
    },
    renderer::{
        error::Result,
        get_camera,
        pipe::{
            pass::{Pass, PassData},
            DepthMode, Effect, NewEffect,
        },
        set_vertex_args, ActiveCamera, Attributes, Camera, Color, Encoder, Factory, Mesh, Normal,
        PosColorNorm, Position, Query, Rgba,
    },
};
use gfx::pso::buffer::ElemStride;
use gfx::Primitive;
use glsl_layout::{mat4, Uniform};
use std::marker::PhantomData;

const LINE_COLOR: Rgba = Rgba(0.2, 0.2, 0.2, 1.0);
const SUBLINE_COLOR: Rgba = Rgba(0.4, 0.4, 0.4, 1.0);

static VERT_SRC: &[u8] = include_bytes!("../../shaders/vertex/origin_grid.glsl");
static GEOM_SRC: &[u8] = include_bytes!("../../shaders/geometry/origin_grid.glsl");
static FRAG_SRC: &[u8] = include_bytes!("../../shaders/fragment/origin_grid.glsl");

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
            },
        }
    }
    true
}

fn new_direction(position: Point3<f32>, direction: Vector3<f32>, color: Rgba) -> GridLine {
    GridLine {
        position: position.into(),
        color: color.into(),
        normal: direction.into(),
    }
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Uniform)]
pub(crate) struct VertexArgs {
    proj: mat4,
    view: mat4,
    model: mat4,
}

/// Grid lines are stored as a position, a direction and a color.
///
/// Storing a direction instead of an end position may not be intuitive,
/// but is similar to other 'VertexFormat's.
pub type GridLine = PosColorNorm;

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
#[derive(Derivative, Clone, Debug)]
#[derivative(Default(bound = "V: Query<(Position, Color, Normal)>"))]
pub struct DrawGridLines<V> {
    _pd: PhantomData<V>,
    mesh: Option<Mesh>,
}

impl<V> DrawGridLines<V>
where
    V: Query<(Position, Color, Normal)>,
{
    /// Create instance of `DrawGridLines` pass
    pub fn new() -> Self {
        DrawGridLines {
            mesh: None,
            ..Default::default()
        }
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
        //WriteStorage<'a, GridLinesComponent>, // GridLines components
        //Option<Write<'a, GridLines>>,         // GridLines resource
        Read<'a, GridLinesParams>,
    );
}

impl<V> Pass for DrawGridLines<V>
where
    V: Query<(Position, Color, Normal)>,
{
    fn compile(&mut self, mut effect: NewEffect) -> Result<Effect> {
        debug!("Building origin grid mesh");
        let mut lines: Vec<GridLine> = Vec::with_capacity(100);
        lines.push(new_direction(
            [0.0, 0.0001, 0.0].into(),
            [0.2, 0.0, 0.0].into(),
            [1.0, 0.0, 0.23, 1.0].into(),
        ));
        lines.push(new_direction(
            [0.0, 0.0, 0.0].into(),
            [0.0, 0.2, 0.0].into(),
            [0.5, 0.85, 0.1, 1.0].into(),
        ));
        lines.push(new_direction(
            [0.0, 0.0001, 0.0].into(),
            [0.0, 0.0, 0.2].into(),
            [0.2, 0.75, 0.93, 1.0].into(),
        ));

        let width: u32 = 10;
        let depth: u32 = 10;

        // Grid lines in X-axis
        for x in 0..=width {
            let (x, width, depth) = (x as f32, width as f32, depth as f32);

            let position = Point3::new(x - width / 2.0, 0.0, -depth / 2.0);
            let direction = Vector3::new(0.0, 0.0, depth);

            lines.push(new_direction(position, direction, LINE_COLOR));

            // Sub-grid lines
            if x != width {
                for sub_x in 1..10 {
                    let sub_offset = Vector3::new((1.0 / 10.0) * sub_x as f32, -0.001, 0.0);
                    lines.push(new_direction(
                        position + sub_offset,
                        direction,
                        SUBLINE_COLOR,
                    ));
                }
            }
        }

        // Grid lines in Z-axis
        for z in 0..=depth {
            let (z, width, depth) = (z as f32, width as f32, depth as f32);

            let position = Point3::new(-width / 2.0, 0.0, z - depth / 2.0);
            let direction = Vector3::new(width, 0.0, 0.0);

            lines.push(new_direction(position, direction, LINE_COLOR));

            // Sub-grid lines
            if z != depth {
                for sub_z in 1..10 {
                    let sub_offset = Vector3::new(0.0, -0.001, (1.0 / 10.0) * sub_z as f32);
                    lines.push(new_direction(
                        position + sub_offset,
                        direction,
                        SUBLINE_COLOR,
                    ));
                }
            }
        }

        self.mesh = Some(Mesh::build(lines).build(&mut effect.factory)?);

        debug!("Building debug lines pass");
        let mut builder = effect.geom(VERT_SRC, GEOM_SRC, FRAG_SRC);

        debug!("Effect compiled, adding vertex/uniform buffers");
        builder.with_raw_vertex_buffer(V::QUERIED_ATTRIBUTES, V::size() as ElemStride, 0);

        builder.with_raw_constant_buffer(
            "VertexArgs",
            std::mem::size_of::<<VertexArgs as Uniform>::Std140>(),
            1,
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
        mut _factory: Factory,
        (active, camera, global, lines_params): <Self as PassData<'a>>::Data,
    ) {
        trace!("Drawing origin grid pass");

        let camera = get_camera(active, &camera, &global);
        effect.update_global(
            "camera_position",
            camera
                .as_ref()
                .map(|&(_, ref trans)| [trans.0[3][0], trans.0[3][1], trans.0[3][2]])
                .unwrap_or([0.0; 3]),
        );

        effect.update_global("line_width", lines_params.line_width);

        let mesh = self
            .mesh
            .as_ref()
            .expect("Failed to get origin mesh reference.");

        if !set_attribute_buffers(effect, &mesh, &[V::QUERIED_ATTRIBUTES]) {
            effect.clear();
            return;
        }

        set_vertex_args(effect, encoder, camera, &GlobalTransform(Matrix4::one()));

        effect.draw(mesh.slice(), encoder);
        effect.clear();
    }
}
