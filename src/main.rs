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

use controls::FirstPersonControlBundle;
use pass::sky::DrawSky;
use tools::pass::grid::DrawGridLines;

struct BaseState;

const CLEAR_COLOR: Rgba = Rgba(0.2, 0.2, 0.2, 1.0);

impl<'a, 'b> SimpleState<'a, 'b> for BaseState {
    fn on_start(&mut self, data: StateData<GameData>) {
        let (width, height) = {
            let dim = data.world.read_resource::<ScreenDimensions>();
            (dim.width(), dim.height())
        };

        // Setup camera
        let mut local_xform = Transform::default();
        local_xform.set_position([0.0, 0.0, 20.0].into());
        data.world
            .create_entity()
            .with(FlyControlTag)
            .with(Camera::from(Projection::perspective(width / height, Deg(72.0))))
            .with(GlobalTransform::default())
            .with(local_xform)
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
