use amethyst::controls::{
    CursorHideSystem, MouseFocusUpdateSystem,
    WindowFocus, HideCursor
};
use amethyst::renderer::{Event, DeviceEvent};
use amethyst::core::bundle::{self, SystemBundle};
use amethyst::core::cgmath::{Deg, Vector3};
use amethyst::ecs::prelude::*;
use amethyst::core::timing::Time;
use amethyst::core::transform::Transform;
use amethyst::shrev::EventChannel;

/// Add this to a camera if you want it to be a player camera.
/// You need to add the FirstPersonControlBundle or the required systems for it to work.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct FirstPersonControlTag;

impl Component for FirstPersonControlTag {
    type Storage = NullStorage<FirstPersonControlTag>;
}

pub struct FirstPersonControlBundle {
    sensitivity_x: f32,
    sensitivity_y: f32,
    speed: f32,
    eye_height: f32,
}

impl FirstPersonControlBundle {
    /// Builds a new first person control bundle using the provided axes as controls.
    pub fn new() -> Self {
        FirstPersonControlBundle {
            sensitivity_x: 1.0,
            sensitivity_y: 1.0,
            speed: 1.0,
            eye_height: 1.0,
        }
    }

    /// Alters the mouse sensitivity on this `FirstPersonControlBundle`
    pub fn with_sensitivity(mut self, x: f32, y: f32) -> Self {
        self.sensitivity_x = x;
        self.sensitivity_y = y;
        self
    }

    /// Alters the speed on this `FirstPersonControlBundle`.
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    /// Alters the eye height on this `FirstPersonControlBundle`.
    pub fn with_eye_height(mut self, eye_height: f32) -> Self {
        self.eye_height = eye_height;
        self
    }
}

impl<'a, 'b> SystemBundle<'a, 'b> for FirstPersonControlBundle {
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> bundle::Result<()> {
        let first_person_movement = FirstPersonMovementSystem::new(
            self.speed,
            self.eye_height,
            self.sensitivity_x,
            self.sensitivity_y,
        );

        let mouse_focus = MouseFocusUpdateSystem::new();
        let cursor_hide = CursorHideSystem::new();

        builder.add(first_person_movement, "first_person_movement", &[]);
        builder.add(mouse_focus, "mouse_focus", &["first_person_movement"]);
        builder.add(cursor_hide, "cursor_hide", &["mouse_focus"]);

        Ok(())
    }
}

/// The system that manages the first person movement.
/// Generic parameters are the parameters for the InputHandler.
pub struct FirstPersonMovementSystem {
    /// The movement speed of the movement in units per second.
    speed: f32,
    eye_height: f32,
    sensitivity_x: f32,
    sensitivity_y: f32,
    movement_dir: Vector3<f32>,
    event_reader: Option<ReaderId<Event>>,
}

impl FirstPersonMovementSystem {
    /// Builds a new `FirstPersonMovementSystem` using the provided
    /// speeds and axis controls.
    pub fn new(
        speed: f32,
        eye_height: f32,
        sensitivity_x: f32,
        sensitivity_y: f32,
    ) -> Self {
        FirstPersonMovementSystem {
            speed,
            eye_height,
            sensitivity_x,
            sensitivity_y,
            movement_dir: Vector3::new(0.0, 0.0, 0.0),
            event_reader: None,
        }
    }
}

impl<'a> System<'a> for FirstPersonMovementSystem {
    type SystemData = (
        Read<'a, Time>,
        Read<'a, EventChannel<Event>>,
        WriteStorage<'a, Transform>,
        ReadStorage<'a, FirstPersonControlTag>,
        Read<'a, WindowFocus>,
        Read<'a, HideCursor>,
    );

    fn run(&mut self, (time, events, mut transform, tag, focus, _hide): Self::SystemData) {
        if !focus.is_focused {
            return;
        }

        for (mut transform, _) in (&mut transform, &tag).join() {
            for event in events.read(&mut self.event_reader.as_mut().unwrap()) {
                match *event {
                    Event::DeviceEvent { ref event, .. } => match *event {
                        DeviceEvent::MouseMotion { delta: (x, y), .. } => {
                            transform.pitch_local(Deg((-1.0) * y as f32 * self.sensitivity_y));
                            transform.yaw_global( Deg((-1.0) * x as f32 * self.sensitivity_x));
                        },
                        DeviceEvent::Key(ref event) => {
                            use VirtualKeyCode::*;
                            use ElementState::*;
                            if let Some(keycode) = event.virtual_keycode {
                                if event.state == Pressed {
                                    match keycode {
                                        W | Up => self.movement_dir.z = -1.0,
                                        S | Down => self.movement_dir.z = 1.0,
                                        A | Left => self.movement_dir.x = -1.0,
                                        D | Right => self.movement_dir.x = 1.0,
                                        _ => (),
                                    }
                                } else {
                                    match keycode {
                                        W | S | Up | Down => self.movement_dir.z = 0.0,
                                        A | D | Left | Right => self.movement_dir.x = 0.0,
                                        _ => (),
                                    }
                                }
                            }
                        }
                        _ => (),
                    },
                    _ => (),
                }
            }

            transform.move_along_local(self.movement_dir, time.delta_seconds() * self.speed);
            // Set the camera position to the eye height. This will change once
            // we add terrain, or add some sort of physics to the character.
            transform.translation.y = self.eye_height;
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        self.event_reader = Some(res.fetch_mut::<EventChannel<Event>>().register_reader());
    }
}
