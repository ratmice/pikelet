use amethyst::controls::{
    CursorHideSystem, FlyControlTag, FreeRotationSystem, MouseFocusUpdateSystem,
};
use amethyst::core::bundle::{self, SystemBundle};
use amethyst::core::cgmath::Vector3;
use amethyst::core::specs::prelude::{
    DispatcherBuilder, Join, Read, ReadStorage, System, WriteStorage,
};
use amethyst::core::timing::Time;
use amethyst::core::transform::Transform;
use amethyst::input::InputHandler;
use std::hash::Hash;
use std::marker::PhantomData;

pub struct FirstPersonControlBundle<A, B> {
    sensitivity_x: f32,
    sensitivity_y: f32,
    speed: f32,
    right_input_axis: Option<A>,
    up_input_axis: Option<A>,
    forward_input_axis: Option<A>,
    _marker: PhantomData<B>,
}

impl<A, B> FirstPersonControlBundle<A, B> {
    /// Builds a new first person control bundle using the provided axes as controls.
    pub fn new(
        right_input_axis: Option<A>,
        up_input_axis: Option<A>,
        forward_input_axis: Option<A>,
    ) -> Self {
        FirstPersonControlBundle {
            sensitivity_x: 1.0,
            sensitivity_y: 1.0,
            speed: 1.0,
            right_input_axis,
            up_input_axis,
            forward_input_axis,
            _marker: PhantomData,
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
}

impl<'a, 'b, A, B> SystemBundle<'a, 'b> for FirstPersonControlBundle<A, B>
where
    A: Send + Sync + Hash + Eq + Clone + 'static,
    B: Send + Sync + Hash + Eq + Clone + 'static,
{
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> bundle::Result<()> {
        let first_person_movement = FirstPersonMovementSystem::<A, B>::new(
            self.speed,
            self.right_input_axis,
            self.up_input_axis,
            self.forward_input_axis,
        );
        let free_rotation = FreeRotationSystem::<A, B>::new(self.sensitivity_x, self.sensitivity_y);
        let mouse_focus = MouseFocusUpdateSystem::new();
        let cursor_hide = CursorHideSystem::new();

        builder.add(first_person_movement, "first_person_movement", &[]);
        builder.add(free_rotation, "free_rotation", &[]);
        builder.add(mouse_focus, "mouse_focus", &["free_rotation"]);
        builder.add(cursor_hide, "cursor_hide", &["mouse_focus"]);

        Ok(())
    }
}

/// The system that manages the first person movement.
/// Generic parameters are the parameters for the InputHandler.
pub struct FirstPersonMovementSystem<A, B> {
    /// The movement speed of the movement in units per second.
    speed: f32,
    /// The name of the input axis to locally move in the x coordinates.
    right_input_axis: Option<A>,
    /// The name of the input axis to locally move in the y coordinates.
    up_input_axis: Option<A>,
    /// The name of the input axis to locally move in the z coordinates.
    forward_input_axis: Option<A>,
    _marker: PhantomData<B>,
}

impl<A, B> FirstPersonMovementSystem<A, B>
where
    A: Send + Sync + Hash + Eq + Clone + 'static,
    B: Send + Sync + Hash + Eq + Clone + 'static,
{
    /// Builds a new `FirstPersonMovementSystem` using the provided speeds and axis controls.
    pub fn new(
        speed: f32,
        right_input_axis: Option<A>,
        up_input_axis: Option<A>,
        forward_input_axis: Option<A>,
    ) -> Self {
        FirstPersonMovementSystem {
            speed,
            right_input_axis,
            up_input_axis,
            forward_input_axis,
            _marker: PhantomData,
        }
    }
}

impl<'a, A, B> System<'a> for FirstPersonMovementSystem<A, B>
where
    A: Send + Sync + Hash + Eq + Clone + 'static,
    B: Send + Sync + Hash + Eq + Clone + 'static,
{
    type SystemData = (
        Read<'a, Time>,
        WriteStorage<'a, Transform>,
        Read<'a, InputHandler<A, B>>,
        // This seems weird, but is required by `FreeRotationSystem` - it might
        // be nice to have our own `FirstPersonControlTag` instead, however
        ReadStorage<'a, FlyControlTag>,
    );

    fn run(&mut self, (time, mut transform, input, tag): Self::SystemData) {
        use amethyst::input;

        let x = input::get_input_axis_simple(&self.right_input_axis, &input);
        let y = input::get_input_axis_simple(&self.up_input_axis, &input);
        let z = input::get_input_axis_simple(&self.forward_input_axis, &input);

        let dir = Vector3::new(x, y, z);

        for (transform, _) in (&mut transform, &tag).join() {
            transform.move_along_local(dir, time.delta_seconds() * self.speed);
        }
    }
}
