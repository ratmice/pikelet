extern crate amethyst;
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate log;
extern crate gfx;
extern crate glsl_layout;

mod pass;
mod controls;
mod tools;

use amethyst::controls::FlyControlTag;
use amethyst::core::cgmath::Deg;
use amethyst::core::transform::{GlobalTransform, Transform, TransformBundle};
use amethyst::input::{is_close_requested, is_key_down, InputBundle};
use amethyst::prelude::*;
use amethyst::renderer::*;
use amethyst::assets::{Loader, AssetStorage};

use controls::FirstPersonControlBundle;
use pass::sky::DrawSky;
use tools::pass::grid::DrawGridLines;

struct BaseState;

const CLEAR_COLOR: Rgba = Rgba(0.2, 0.2, 0.2, 1.0);
const GROUND_COLOR: [f32;4] = [0.47, 0.53, 0.49, 1.0];
const FOV: Deg<f32> = Deg(60.0);


struct MeshLibrary {
    cube: MeshHandle,
    plane_sm: MeshHandle,
    plane_md: MeshHandle,
    plane_lg: MeshHandle,
}

impl MeshLibrary {
    fn new(world: &mut World) -> MeshLibrary {
        let loader = world.read_resource::<Loader>();
        let meshes: &AssetStorage<Mesh> = &world.read_resource();

        let cube = {
            let verts = Shape::Cube.generate::<Vec<PosNormTex>>(None);
            loader.load_from_data(verts, (), meshes)
        };

        let plane_sm = {
            let verts = Shape::Plane(None).generate::<Vec<PosNormTex>>(None);
            loader.load_from_data(verts, (), meshes)
        };

        let plane_md = {
            let verts = Shape::Plane(Some((4, 4)))
                .generate::<Vec<PosNormTex>>(Some((10.0, 10.0, 10.0)));
            loader.load_from_data(verts, (), meshes)
        };

        let plane_lg = {
            let verts = Shape::Plane(Some((10,10)))
                .generate::<Vec<PosNormTex>>(Some((100.0, 100.0, 100.0)));
            loader.load_from_data(verts, (), meshes)
        };

        MeshLibrary {
            cube,
            plane_sm,
            plane_md,
            plane_lg,
        }
    }
}

struct MaterialLibrary {
    white: Material,
    green_a: Material,
}

impl MaterialLibrary {
    fn new(world: &mut World) -> MaterialLibrary {
        let loader = world.read_resource::<Loader>();

        let textures: &AssetStorage<Texture> = &world.read_resource();
        let default_material = world.read_resource::<MaterialDefaults>().0.clone();

        let white_albedo = loader.load_from_data([1.0, 1.0, 1.0, 1.0].into(), (), textures);
        let ground_albedo = loader.load_from_data(GROUND_COLOR.into(), (), textures);

        MaterialLibrary {
            white: Material {
                albedo: white_albedo,
                ..default_material.clone()
            },
            green_a: Material {
                albedo: ground_albedo,
                ..default_material.clone()
            }
        }
    }
}

fn initialize_ground(
    world: &mut World,
    meshes: &MeshLibrary,
    materials: &MaterialLibrary
) {
    let mut ground_xform = Transform::default();
    ground_xform.set_position([0.0, -0.001, 0.0].into());
    ground_xform.pitch_local(Deg(-90.0));
    world.create_entity()
        .with(GlobalTransform::default())
        .with(ground_xform)
        .with(meshes.plane_lg.clone())
        .with(materials.green_a.clone())
        .build();
}

fn initialize_house(
    world: &mut World,
    meshes: &MeshLibrary,
    materials: &MaterialLibrary
) {
    world.create_entity()
        .with(GlobalTransform::default())
        .with(meshes.cube.clone())
        .with(materials.white.clone())
        .build();
}

fn initialize_camera(world: &mut World) {
    let (width, height) = {
        let dim = world.read_resource::<ScreenDimensions>();
        (dim.width(), dim.height())
    };

    // Setup camera
    let mut cam_xform = Transform::default();
    cam_xform.set_position([0.0, 0.0, 20.0].into());
    world
        .create_entity()
        .with(FlyControlTag)
        .with(Camera::from(Projection::perspective(width / height, FOV)))
        .with(GlobalTransform::default())
        .with(cam_xform)
        .build();
}

impl<'a, 'b> SimpleState<'a, 'b> for BaseState {
    fn on_start(&mut self, data: StateData<GameData>) {
        let StateData { world, .. } = data;

        let mesh_lib = MeshLibrary::new(world);
        let mat_lib = MaterialLibrary::new(world);

        initialize_camera(world);

        initialize_ground(world, &mesh_lib, &mat_lib);
        initialize_house(world, &mesh_lib, &mat_lib);
    }

    fn handle_event(
        &mut self,
        _data: StateData<GameData>,
        event: StateEvent,
    ) -> SimpleTrans<'a, 'b> {
        if let StateEvent::Window(event) = &event {
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                Trans::Quit
            } else {
                Trans::None
            }
        } else {
            Trans::None
        }
    }
}

fn main() -> amethyst::Result<()> {
    use amethyst::LoggerConfig;

    amethyst::start_logger(LoggerConfig::default());

    let app_root = amethyst::utils::application_root_dir();

    let input_bundle = {
        let path = format!("{}/resources/input.ron", app_root);
        InputBundle::<String, String>::new().with_bindings_from_file(path)?
    };

    let first_person_control_bundle = FirstPersonControlBundle::<String, String>::new(
        Some("move_x".to_owned()),
        Some("move_z".to_owned()),
    )
    .with_sensitivity(0.15, 0.15)
    .with_speed(13.8) // average walking speed of a human at 13.8 dm / sec
    .with_eye_height(10.0);

    let transform_bundle =
        TransformBundle::new().with_dep(&["first_person_movement", "free_rotation"]);

    let render_bundle = {
        let display_config = {
            let path = format!("{}/resources/display_config.ron", app_root);
            DisplayConfig::load(&path)
        };

        let pipe = Pipeline::build().with_stage(
            Stage::with_backbuffer()
                .clear_target(CLEAR_COLOR, 1.0)
                .with_pass(DrawSky::<PosNormTex>::new())
                .with_pass(DrawGridLines::<PosColorNorm>::new())
                .with_pass(DrawFlat::<PosNormTex>::new()),
        );

        RenderBundle::new(pipe, Some(display_config))
    };

    let game_data = GameDataBuilder::default()
        .with_bundle(input_bundle)?
        .with_bundle(first_person_control_bundle)?
        .with_bundle(transform_bundle)?
        .with_bundle(render_bundle)?;

    let mut game = Application::new(app_root, BaseState, game_data)?;
    game.run();

    Ok(())
}
