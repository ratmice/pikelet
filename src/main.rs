extern crate amethyst;

use amethyst::{
    controls::{FlyControlBundle, FlyControlTag},
    core::{
        cgmath::{Deg, Point3, Vector3},
        transform::{GlobalTransform, Transform, TransformBundle},
    },
    ecs::{Read, System, Write},
    input::{is_close_requested, is_key_down, InputBundle},
    prelude::*,
    renderer::*,
    utils::application_root_dir,
};

struct BaseState;

impl<'a, 'b> SimpleState<'a, 'b> for BaseState {
    fn on_start(&mut self, data: StateData<GameData>) {
        data.world
            .add_resource(DebugLines::new().with_capacity(100));
        data.world.add_resource(DebugLinesParams {
            line_width: 1.0 / 400.0,
        });

        // Setup debug lines as a component and add lines to render axis&grid
        let mut debug_lines_component = DebugLinesComponent::new().with_capacity(100);
        debug_lines_component.add_direction(
            [0.0, 0.0001, 0.0].into(),
            [0.2, 0.0, 0.0].into(),
            [1.0, 0.0, 0.23, 1.0].into(),
        );
        debug_lines_component.add_direction(
            [0.0, 0.0, 0.0].into(),
            [0.0, 0.2, 0.0].into(),
            [0.5, 0.85, 0.1, 1.0].into(),
        );
        debug_lines_component.add_direction(
            [0.0, 0.0001, 0.0].into(),
            [0.0, 0.0, 0.2].into(),
            [0.2, 0.75, 0.93, 1.0].into(),
        );

        let width: u32 = 10;
        let depth: u32 = 10;
        let main_color = [0.4, 0.4, 0.4, 1.0].into();

        // Grid lines in X-axis
        for x in 0..=width {
            let (x, width, depth) = (x as f32, width as f32, depth as f32);

            let position = Point3::new(x - width / 2.0, 0.0, -depth / 2.0);
            let direction = Vector3::new(0.0, 0.0, depth);

            debug_lines_component.add_direction(position, direction, main_color);

            // Sub-grid lines
            if x != width {
                for sub_x in 1..10 {
                    let sub_offset = Vector3::new((1.0 / 10.0) * sub_x as f32, -0.001, 0.0);

                    debug_lines_component.add_direction(
                        position + sub_offset,
                        direction,
                        [0.1, 0.1, 0.1, 0.1].into(),
                    );
                }
            }
        }

        // Grid lines in Z-axis
        for z in 0..=depth {
            let (z, width, depth) = (z as f32, width as f32, depth as f32);

            let position = Point3::new(-width / 2.0, 0.0, z - depth / 2.0);
            let direction = Vector3::new(width, 0.0, 0.0);

            debug_lines_component.add_direction(position, direction, main_color);

            // Sub-grid lines
            if z != depth {
                for sub_z in 1..10 {
                    let sub_offset = Vector3::new(0.0, -0.001, (1.0 / 10.0) * sub_z as f32);

                    debug_lines_component.add_direction(
                        position + sub_offset,
                        direction,
                        [0.1, 0.1, 0.1, 0.0].into(),
                    );
                }
            }
        }

        data.world.register::<DebugLinesComponent>();
        data.world
            .create_entity()
            .with(debug_lines_component)
            .build();

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
        data: StateData<GameData>,
        event: StateEvent,
    ) -> SimpleTrans<'a, 'b> {
        let StateData { world, .. } = data;
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
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir();

    let input_bundle = {
        let path = format!("{}/resources/input.ron", app_root);
        InputBundle::<String, String>::new().with_bindings_from_file(path)?
    };

    let fly_control_bundle = FlyControlBundle::<String, String>::new(
        Some(String::from("move_x")),
        Some(String::from("move_y")),
        Some(String::from("move_z")),
    )
    .with_sensitivity(0.1, 0.1);

    let transform_bundle = TransformBundle::new().with_dep(&["fly_movement"]);

    let render_bundle = {
        let display_config = {
            let path = format!("{}/resources/display_config.ron", app_root);
            DisplayConfig::load(&path)
        };

        let pipe = Pipeline::build().with_stage(
            Stage::with_backbuffer()
                .clear_target([0.05, 0.05, 0.05, 1.0], 1.0)
                .with_pass(DrawFlat::<PosNormTex>::new())
                .with_pass(DrawDebugLines::<PosColorNorm>::new()),
        );

        RenderBundle::new(pipe, Some(display_config))
    };

    let game_data = GameDataBuilder::default()
        .with_bundle(input_bundle)?
        .with_bundle(fly_control_bundle)?
        .with_bundle(transform_bundle)?
        .with_bundle(render_bundle)?;

    let mut game = Application::new(app_root, BaseState, game_data)?;
    game.run();

    Ok(())
}
