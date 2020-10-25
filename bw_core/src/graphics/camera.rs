use amethyst::{
    core::Transform,
    ecs::{Entities, Join, Read, ReadStorage, System, WriteStorage},
    input::{InputHandler, StringBindings},
    renderer::camera::{ActiveCamera, Camera},
};

#[derive(Default)]
pub struct CameraMovementSystem;

impl<'s> System<'s> for CameraMovementSystem {
    type SystemData = (
        Read<'s, ActiveCamera>,
        Entities<'s>,
        ReadStorage<'s, Camera>,
        WriteStorage<'s, Transform>,
        Read<'s, InputHandler<StringBindings>>,
    );

    fn run(&mut self, (active_camera, entities, cameras, mut transforms, input): Self::SystemData) {
        let x_move = input.axis_value("camera_x").unwrap_or(0.0);
        let y_move = input.axis_value("camera_y").unwrap_or(0.0);

        if x_move != 0.0 || y_move != 0.0 {
            let mut camera_join = (&cameras, &mut transforms).join();
            if let Some((_, camera_transform)) = active_camera
                .entity
                .and_then(|a| camera_join.get(a, &entities))
                .or_else(|| camera_join.next())
            {
                camera_transform.prepend_translation_x(x_move * 50.0);
                camera_transform.prepend_translation_y(y_move * 50.0);
            }
        }
    }
}

#[derive(Default)]
pub struct CameraRotationSystem;

impl<'s> System<'s> for CameraRotationSystem {
    type SystemData = (
        Read<'s, ActiveCamera>,
        Entities<'s>,
        ReadStorage<'s, Camera>,
        WriteStorage<'s, Transform>,
        Read<'s, InputHandler<StringBindings>>,
    );

    fn run(&mut self, (active_camera, entities, cameras, mut transforms, input): Self::SystemData) {
        let x_rotate = input.axis_value("camera_rx").unwrap_or(0.0);
        let y_rotate = input.axis_value("camera_ry").unwrap_or(0.0);
        let z_rotate = input.axis_value("camera_rz").unwrap_or(0.0);

        if x_rotate != 0.0 || y_rotate != 0.0 || z_rotate != 0.0 {
            let mut camera_join = (&cameras, &mut transforms).join();
            if let Some((_, camera_transform)) = active_camera
                .entity
                .and_then(|a| camera_join.get(a, &entities))
                .or_else(|| camera_join.next())
            {
                camera_transform.prepend_rotation_x_axis(x_rotate.to_radians());
                camera_transform.prepend_rotation_y_axis(y_rotate.to_radians());
                camera_transform.prepend_rotation_z_axis(z_rotate.to_radians());
            }
        }
    }
}
