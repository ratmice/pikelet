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
use amethyst::assets::Loader;

use controls::FirstPersonControlBundle;
use pass::sky::DrawSky;
use tools::pass::grid::DrawGridLines;

struct BaseState;

const CLEAR_COLOR: Rgba = Rgba(0.2, 0.2, 0.2, 1.0);

impl<'a, 'b> SimpleState<'a, 'b> for BaseState {
    fn on_start(&mut self, data: StateData<GameData>) {
        let StateData { world, .. } = data;

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
            .with(Camera::from(Projection::perspective(width / height, Deg(72.0))))
            .with(GlobalTransform::default())
            .with(cam_xform)
            .build();

        // Setup ground plane
        let (mesh, albedo) = {
            let loader = world.read_resource::<Loader>();

            let meshes = &world.read_resource();
            let textures = &world.read_resource();

            let verts = Shape::Plane(Some((10,10)))
                .generate::<Vec<PosNormTex>>(Some((100.0, 100.0, 100.0)));
            let mesh: MeshHandle = loader.load_from_data(verts, (), meshes);

            let albedo = loader.load_from_data([0.27, 0.43, 0.29, 1.0].into(), (), textures);

            (mesh, albedo)
        };

        let mat_defaults = world.read_resource::<MaterialDefaults>().0.clone();
        let mtl = Material {
            albedo: albedo.clone(),
            ..mat_defaults.clone()
        };

        let mut ground_xform = Transform::default();
        ground_xform.set_position([0.0, -0.001, 0.0].into());
        ground_xform.pitch_local(Deg(-90.0));
        world
            .create_entity()
            .with(GlobalTransform::default())
            .with(ground_xform)
            .with(mesh.clone())
            .with(mtl)
            .build();
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
