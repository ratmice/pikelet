//! Grid lines pass

use amethyst::{
    core::{
        cgmath::{Matrix4, One},
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
        set_vertex_args, ActiveCamera, Attributes, Camera, Encoder, Factory, Mesh, Normal,
        PosNormTex, Position, Query, Shape, TexCoord, VertexFormat, Rgba,
    },
};
use gfx::pso::buffer::ElemStride;
use glsl_layout::{mat4, Uniform};
use std::marker::PhantomData;

static VERT_SRC: &[u8] = include_bytes!("../shaders/vertex/sky.glsl");
static FRAG_SRC: &[u8] = include_bytes!("../shaders/fragment/sky.glsl");

#[derive(Clone, Debug)]
pub struct SkyColors {
    pub zenith: Rgba,
    pub nadir: Rgba,
}

impl Default for SkyColors {
    fn default() -> SkyColors {
        SkyColors {
            zenith: Rgba(0.75, 1.0, 1.0, 1.0),
            nadir: Rgba(0.4, 0.6, 0.65, 1.0),
        }
    }
}

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
    _marker: PhantomData<V>,
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
            ..DrawSky::default()
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
        Read<'a, SkyColors>
    );
}

impl<V> Pass for DrawSky<V>
where
    V: Query<(Position, Normal, TexCoord)>,
{
    fn compile(&mut self, mut effect: NewEffect) -> Result<Effect> {
        let verts = Shape::Cube.generate_vertices::<Vec<PosNormTex>>(None);
        self.mesh = Some(Mesh::build(verts).build(&mut effect.factory)?);

        debug!("Building sky pass");
        effect
            .simple(VERT_SRC, FRAG_SRC)
            .without_back_face_culling()
            .with_raw_constant_buffer(
                "VertexArgs",
                std::mem::size_of::<<VertexArgs as Uniform>::Std140>(),
                1
            )
            .with_raw_vertex_buffer(
                PosNormTex::ATTRIBUTES, PosNormTex::size() as ElemStride, 0
            )
            .with_raw_global("camera_position")
            .with_raw_global("zenith_color")
            .with_raw_global("nadir_color")
            .with_output("color", Some(DepthMode::LessEqualWrite))
            .build()
    }

    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoder: &mut Encoder,
        effect: &mut Effect,
        mut _factory: Factory,
        (active, camera, global, colors): <Self as PassData<'a>>::Data,
    ) {
        trace!("Drawing origin grid pass");

        let camera = get_camera(active, &camera, &global);

        let mesh = self
            .mesh
            .as_ref()
            .expect("Sky effect is not compiled!");

        set_vertex_args(effect, encoder, camera, &GlobalTransform(Matrix4::one()));

        if !set_attribute_buffers(effect, &mesh, &[V::QUERIED_ATTRIBUTES]) {
            effect.clear();
            return;
        }

        effect.update_global("zenith_color", Into::<[f32;3]>::into(colors.zenith));
        effect.update_global("nadir_color", Into::<[f32;3]>::into(colors.nadir));
        effect.draw(mesh.slice(), encoder);
        effect.clear();
    }
}
