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
        set_vertex_args, ActiveCamera, Camera, Color, Encoder, Factory, Mesh,
        Normal, Position, TexCoord, Query, Attributes, Rgba, PosNormTex, Shape,
        VertexFormat
    },
};
use gfx::pso::buffer::ElemStride;
use gfx::Primitive;
use glsl_layout::{Uniform, mat4};
use std::marker::PhantomData;


static VERT_SRC: &[u8] = include_bytes!("../shaders/vertex/sky.glsl");
static FRAG_SRC: &[u8] = include_bytes!("../shaders/fragment/sky.glsl");


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


/// Draw a simple origin grid to aid in view orientation
///
/// # Type Parameters:
///
/// * `V`: `VertexFormat`
#[derive(Derivative, Clone, Debug)]
#[derivative(Default(bound = "V: Query<(Position, Normal, TexCoord)>"))]
pub struct DrawSky<V> {
    _pd: PhantomData<V>,
    mesh: Option<Mesh>,
}

impl<V> DrawSky<V>
    where
        V: Query<(Position, Normal, TexCoord)>,
{
    /// Create instance of `DrawSky` pass
    pub fn new() -> Self {
        DrawSky {
            mesh: None,
            .. Default::default()
        }
    }
}

impl<'a, V> PassData<'a> for DrawSky<V>
    where
        V: Query<(Position, Normal, TexCoord)>,
{
    type Data = (
        Option<Read<'a, ActiveCamera>>,
        ReadStorage<'a, Camera>,
        ReadStorage<'a, GlobalTransform>,
    );
}

impl<V> Pass for DrawSky<V>
    where
        V: Query<(Position, Normal, TexCoord)>,
{
    fn compile(
        &mut self,
        mut effect: NewEffect
    ) -> Result<Effect> {
        let verts = Shape::Cube.generate_vertices::<Vec<PosNormTex>>(Some((-1.,-1.,-1.)));
        self.mesh = Some(Mesh::build(verts).build(&mut effect.factory)?);

        debug!("Building debug lines pass");
        effect
            .simple(VERT_SRC, FRAG_SRC)
            .with_raw_constant_buffer(
                "VertexArgs",
                std::mem::size_of::<<VertexArgs as Uniform>::Std140>(),
                1
            )
            .with_raw_vertex_buffer(
                PosNormTex::ATTRIBUTES, PosNormTex::size() as ElemStride, 0
            )
            .with_raw_global("camera_position")
            //.with_primitive_type(Primitive::PointList)
            .with_output("color", Some(DepthMode::LessEqualTest))
            .build()
    }

    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoder: &mut Encoder,
        effect: &mut Effect,
        mut _factory: Factory,
        (active, camera, global): <Self as PassData<'a>>::Data,
    ) {
        trace!("Drawing origin grid pass");

        let camera = get_camera(active, &camera, &global);

        let mesh = self.mesh.as_ref()
            .expect("Failed to get origin mesh reference.");

        set_vertex_args(effect, encoder, camera, &GlobalTransform(Matrix4::one()));

        if !set_attribute_buffers(effect, &mesh, &[V::QUERIED_ATTRIBUTES]) {
            effect.clear();
            return;
        }

        effect.draw(mesh.slice(), encoder);
        effect.clear();
    }
}
