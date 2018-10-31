extern crate amethyst;
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate log;
extern crate gfx;
extern crate glsl_layout;

mod controls;
mod tools;

use amethyst::controls::FlyControlTag;
use amethyst::core::cgmath::Deg;
use amethyst::core::transform::{GlobalTransform, Transform, TransformBundle};
use amethyst::input::{is_close_requested, is_key_down, InputBundle};
use amethyst::prelude::*;
use amethyst::renderer::*;

use controls::FirstPersonControlBundle;
use tools::pass::grid::DrawGridLines;

struct BaseState;

const SKY_COLOR: Rgba = Rgba(0.4, 0.6, 0.6, 1.0);

impl<'a, 'b> SimpleState<'a, 'b> for BaseState {
    fn on_start(&mut self, data: StateData<GameData>) {
        // Setup camera
        // TODO: need to use window dimensions for correct projection
        let mut local_xform = Transform::default();
        local_xform.set_position([0.0, 0.5, 2.0].into());
        data.world
            .create_entity()
            .with(FlyControlTag)
            .with(Camera::from(Projection::perspective(1.333, Deg(72.0))))
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
        Some("move_y".to_owned()),
        Some("move_z".to_owned()),
    )
    .with_sensitivity(0.1, 0.1)
    .with_speed(1.0);

    let transform_bundle =
        TransformBundle::new().with_dep(&["first_person_movement", "free_rotation"]);

    let render_bundle = {
        let display_config = {
            let path = format!("{}/resources/display_config.ron", app_root);
            DisplayConfig::load(&path)
        };

        let pipe = Pipeline::build().with_stage(
            Stage::with_backbuffer()
                .clear_target(SKY_COLOR, 1.0)
                //.with_pass(DrawFlat::<PosNormTex>::new())
                .with_pass(DrawGridLines::<PosColorNorm>::new()),
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
