extern crate amethyst;
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate log;
extern crate gfx;
extern crate glsl_layout;
extern crate rand;

mod pass;
mod controls;
mod tools;

use amethyst::controls::FlyControlTag;
use amethyst::core::cgmath::Deg;
use amethyst::core::transform::{GlobalTransform, Transform, TransformBundle};
use amethyst::core::Parent;
use amethyst::input::{is_close_requested, is_key_down, InputBundle};
use amethyst::prelude::*;
use amethyst::renderer::*;
use amethyst::assets::{Loader, AssetStorage};
use rand::prelude::*;

use controls::FirstPersonControlBundle;
use pass::sky::DrawSky;
use tools::pass::grid::DrawGridLines;

struct BaseState;

const CLEAR_COLOR: Rgba = Rgba(0.2, 0.2, 0.2, 1.0);
const GROUND_COLOR: [f32;4] = [0.47, 0.53, 0.49, 1.0];
const FOV: Deg<f32> = Deg(60.0);


struct MeshLibrary {
    cube: MeshHandle,
    sphere: MeshHandle,
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

        let sphere = {
            let verts = Shape::IcoSphere(None).generate::<Vec<PosNormTex>>(None);
            loader.load_from_data(verts, (), meshes)
        };

        let plane_sm = {
            let verts = Shape::Plane(None).generate::<Vec<PosNormTex>>(None);
            loader.load_from_data(verts, (), meshes)
        };

        let plane_md = {
            let verts = Shape::Plane(Some((10, 10)))
                .generate::<Vec<PosNormTex>>(None);
            loader.load_from_data(verts, (), meshes)
        };

        let plane_lg = {
            let verts = Shape::Plane(Some((100,100)))
                .generate::<Vec<PosNormTex>>(None);
            loader.load_from_data(verts, (), meshes)
        };

        MeshLibrary {
            cube,
            sphere,
            plane_sm,
            plane_md,
            plane_lg,
        }
    }
}

struct MaterialLibrary {
    white: Material,
    dark_grey: Material,
    green_a: Material,
}

impl MaterialLibrary {
    fn new(world: &mut World) -> MaterialLibrary {
        let loader = world.read_resource::<Loader>();

        let textures: &AssetStorage<Texture> = &world.read_resource();
        let default_material = world.read_resource::<MaterialDefaults>().0.clone();

        let white_albedo = loader.load_from_data([1.0, 1.0, 1.0, 1.0].into(), (), textures);
        let dark_grey_albedo = loader.load_from_data([0.3, 0.3, 0.3, 1.0].into(), (), textures);
        let ground_albedo = loader.load_from_data(GROUND_COLOR.into(), (), textures);

        MaterialLibrary {
            white: Material {
                albedo: white_albedo,
                ..default_material.clone()
            },
            dark_grey: Material {
                albedo: dark_grey_albedo,
                ..default_material.clone()
            },
            green_a: Material {
                albedo: ground_albedo,
                ..default_material.clone()
            }
        }
    }
}

fn initialize_ground(world: &mut World) {
    let plane = world.read_resource::<MeshLibrary>().plane_lg.clone();
    let mtl = world.read_resource::<MaterialLibrary>().green_a.clone();
    let mut xform = Transform::default();
    xform.set_position([0.0, -0.001, 0.0].into());
    xform.pitch_local(Deg(-90.0));
    xform.scale.x = 1000.0;
    xform.scale.y = 1000.0;
    world.create_entity()
        .with(xform)
        .with(plane)
        .with(mtl)
        .build();
}

fn initialize_house(world: &mut World) {
    let cube = world.read_resource::<MeshLibrary>().cube.clone();
    let mtl = world.read_resource::<MaterialLibrary>().white.clone();

    let house_w = 20.0;
    let house_h = 15.0;
    let house_d = 20.0;

    let mut xform = Transform::default();
    xform.scale.x = house_w;
    xform.scale.z = house_d;
    xform.scale.y = house_h;
    xform.set_position([0.0, house_h, -(house_d + 5.0)].into());
    world.create_entity()
        .with(xform)
        .with(cube)
        .with(mtl)
        .build();
}

fn initialize_tree(world: &mut World, root_xform: Transform, has_leaves: bool) {
    let (trunk_mesh, trunk_mtl, leaves_mesh, leaves_mtl) = {
        let meshes = world.read_resource::<MeshLibrary>();
        let materials = world.read_resource::<MaterialLibrary>();

        (meshes.cube.clone(), materials.dark_grey.clone(),
         meshes.sphere.clone(), materials.green_a.clone())
    };

    let root = {
        world.create_entity()
            .with(GlobalTransform::default())
            .with(root_xform)
            .build()
    };

    let trunk_diameter = 0.5;
    let trunk_height = 10.0;
    let mut trunk_xform = Transform::default();
    trunk_xform.scale.x = trunk_diameter;
    trunk_xform.scale.y = trunk_height;
    trunk_xform.scale.z = trunk_diameter;
    trunk_xform.set_position([0.0, trunk_height, 0.0].into());
    world.create_entity()
        .with(Parent { entity: root })
        .with(GlobalTransform::default())
        .with(trunk_xform)
        .with(trunk_mesh)
        .with(trunk_mtl)
        .build();

    if has_leaves {
        let mut leaves_xform = Transform::default();
        leaves_xform.scale.x = 10.0;
        leaves_xform.scale.y = 10.0;
        leaves_xform.scale.z = 10.0;
        leaves_xform.set_position([0.0, trunk_height * 2.0, 0.0].into());
        world.create_entity()
            .with(Parent { entity: root })
            .with(GlobalTransform::default())
            .with(leaves_xform)
            .with(leaves_mesh)
            .with(leaves_mtl)
            .build();
    }
}

fn initialize_forest(world: &mut World) {
    let mut rng = thread_rng();
    for _ in 0..40 {
        let mut xform = Transform::default();
        let x_range = (-200.0, 200.0);
        let z_range = (20.0, 800.0);
        let scale_range = (1.0, 3.0);

        let x = rng.gen_range(x_range.0, x_range.1);
        let z = rng.gen_range(z_range.0, z_range.1);
        xform.set_position([x, 0.0, -z].into());

        xform.scale *= rng.gen_range(scale_range.0, scale_range.1);

        initialize_tree(world, xform, true);
    }
}

fn initialize_lights(world: &mut World) {
    world.write_resource::<AmbientColor>().0 = [0.75,1.0,1.0,1.0].into();

    let sunlight: Light = DirectionalLight {
        color: Rgba::white(),
        direction: [0.0, -1.0, -1.0],
    }.into();

    world.create_entity()
        .with(sunlight)
        .build();
}

fn initialize_camera(world: &mut World) {
    let (width, height) = {
        let dim = world.read_resource::<ScreenDimensions>();
        (dim.width(), dim.height())
    };

    // Setup camera
    let mut cam_xform = Transform::default();
    cam_xform.set_position([0.0, 0.0, 40.0].into());
    world
        .create_entity()
        .with(FlyControlTag)
        .with(Camera::from(Projection::perspective(width / height, FOV)))
        .with(GlobalTransform::default())
        .with(cam_xform)
        .build();
}

fn initialize_object_libraries(world: &mut World) {
    let mesh_lib = MeshLibrary::new(world);
    let mat_lib = MaterialLibrary::new(world);

    world.add_resource(mesh_lib);
    world.add_resource(mat_lib);
}

impl<'a, 'b> SimpleState<'a, 'b> for BaseState {
    fn on_start(&mut self, data: StateData<GameData>) {
        let StateData { world, .. } = data;

        initialize_object_libraries(world);

        initialize_camera(world);
        initialize_lights(world);
        initialize_ground(world);
        initialize_house(world);
        initialize_forest(world);
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
                .with_pass(DrawShaded::<PosNormTex>::new()),
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
