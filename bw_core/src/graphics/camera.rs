use amethyst::{
    assets::{AssetStorage, Handle},
    core::math::max,
    core::Transform,
    core::{math::min, num::Float},
    ecs::{Entities, Join, Read, ReadStorage, System, WriteStorage},
    input::{InputHandler, StringBindings},
    renderer::camera::{ActiveCamera, Camera},
};
use bw_assets::map::Map;

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

#[derive(Default)]
pub struct CameraTranslationClampSystem;

impl<'s> System<'s> for CameraTranslationClampSystem {
    type SystemData = (
        Read<'s, ActiveCamera>,
        Entities<'s>,
        ReadStorage<'s, Camera>,
        WriteStorage<'s, Transform>,
        Read<'s, AssetStorage<Map>>,
        Read<'s, Option<Handle<Map>>>,
    );

    fn run(
        &mut self,
        (active_camera_entity, entities, cameras, mut transforms,map_storage, map_handle_opt): Self::SystemData,
    ) {
        let mut camera_join = (&cameras, &mut transforms).join();
        if let (Some((active_camera, camera_transform)), Some(map)) = (
            active_camera_entity
                .entity
                .and_then(|a| camera_join.get(a, &entities))
                .or_else(|| camera_join.next()),
            map_handle_opt
                .as_ref()
                .and_then(|map_handle| map_storage.get(&map_handle.clone())),
        ) {
            let camera_width = 2.0 / active_camera.matrix[(0, 0)];
            let camera_height = -2.0 / active_camera.matrix[(1, 1)];

            let x_clamped = min(
                max(
                    camera_transform.translation_mut().x as i32,
                    -(map.pixel_width() as i32) / 2,
                ),
                ((map.pixel_width() as i32) / 2) - Float::ceil(camera_width) as i32,
            ) as f32;

            let y_clamped = min(
                max(
                    camera_transform.translation_mut().y as i32,
                    (-(map.pixel_height() as i32) / 2) + Float::ceil(camera_height) as i32,
                ),
                (map.pixel_height() as i32) / 2,
            ) as f32;

            camera_transform.translation_mut().x = x_clamped;
            camera_transform.translation_mut().y = y_clamped;
        }
    }
}
