use std::ops::Range;

use super::{Minimap, MinimapMarker};
use amethyst::{
    assets::{AssetStorage, Handle},
    core::{
        ecs::{Read, ReadExpect, ReadStorage, System, WriteStorage},
        Transform,
    },
    input::{InputHandler, StringBindings},
    renderer::ActiveCamera,
    renderer::Camera,
    ui::{Anchor, UiTransform},
    {window::ScreenDimensions, winit::MouseButton},
};
use bw_assets::map::Map;

#[derive(Default)]

pub struct MinimapMarkerCameraTrackingSystem;

impl<'s> System<'s> for MinimapMarkerCameraTrackingSystem {
    type SystemData = (
        Read<'s, ActiveCamera>,
        Read<'s, MinimapMarker>,
        Read<'s, Option<Handle<Map>>>,
        Read<'s, AssetStorage<Map>>,
        ReadStorage<'s, Transform>,
        WriteStorage<'s, UiTransform>,
    );

    fn run(
        &mut self,
        (
            active_camera_entity,
            minimap_marker,
            map_handle_opt,
            map_storage,
            transforms,
            mut ui_transforms,
        ): Self::SystemData,
    ) {
        let minimap_marker_transform_opt = minimap_marker
            .entity()
            .and_then(|entity| ui_transforms.get_mut(entity));
        let camera_transform_opt = active_camera_entity
            .entity
            .and_then(|entity| transforms.get(entity));
        let map_opt = map_handle_opt
            .as_ref()
            .and_then(|map_handle| map_storage.get(&map_handle.clone()));

        if let (Some(minimap_marker_transform), Some(camera_transform), Some(map)) =
            (minimap_marker_transform_opt, camera_transform_opt, map_opt)
        {
            let x_percentage = normalize(
                camera_transform.translation().x as f32,
                -((map.pixel_width() as f32) / 2.0),
                (map.pixel_width() / 2) as f32,
            );
            let y_percentage = normalize(
                camera_transform.translation().y as f32,
                -((map.pixel_height() as f32) / 2.0),
                (map.pixel_height() / 2) as f32,
            );

            let minimap_x = (map.tile_width() as f32) * x_percentage;
            let minimap_y = (map.tile_height() as f32) * y_percentage;

            minimap_marker_transform.local_x = minimap_x;
            minimap_marker_transform.local_y = minimap_y;
        }
    }
}
/// System that keeps track of mouse movements on the minimap
pub struct MinimapMouseMovementTrackingSystem {
    was_lmb_originally_pressed_in_minimap_box: bool,
}

impl Default for MinimapMouseMovementTrackingSystem {
    fn default() -> Self {
        MinimapMouseMovementTrackingSystem {
            was_lmb_originally_pressed_in_minimap_box: false,
        }
    }
}

impl<'s> System<'s> for MinimapMouseMovementTrackingSystem {
    type SystemData = (
        Read<'s, InputHandler<StringBindings>>,
        Read<'s, Minimap>,
        WriteStorage<'s, UiTransform>,
        ReadExpect<'s, ScreenDimensions>,
        Read<'s, ActiveCamera>,
        ReadStorage<'s, Camera>,
        WriteStorage<'s, Transform>,
        Read<'s, AssetStorage<Map>>,
        Read<'s, Option<Handle<Map>>>,
    );

    fn run(
        &mut self,
        (
            input,
            minimap_opt,
            mut ui_transforms,
            screen_dimensions,
            active_camera_entity,
            cameras,
            mut transforms,
            map_storage,
            map_handle_opt,
        ): Self::SystemData,
    ) {
        let mouse_position_opt = input.mouse_position();
        let is_lmb_down = input.mouse_button_is_down(MouseButton::Left);
        let minimap_ui_transform_opt = minimap_opt
            .entity()
            .and_then(|entity| ui_transforms.get_mut(entity));
        let camera_transform_opt = active_camera_entity
            .entity
            .and_then(|entity| transforms.get_mut(entity));
        let map_opt = map_handle_opt
            .as_ref()
            .and_then(|map_handle| map_storage.get(&map_handle.clone()));
        let active_camera_opt = active_camera_entity
            .entity
            .and_then(|entity| cameras.get(entity));

        if !is_lmb_down {
            self.was_lmb_originally_pressed_in_minimap_box = false;
        }

        if let (
            Some(mouse_position),
            Some(minimap_ui_transform),
            Some(camera_transform),
            Some(map),
            Some(active_camera),
        ) = (
            mouse_position_opt,
            minimap_ui_transform_opt,
            camera_transform_opt,
            map_opt,
            active_camera_opt,
        ) {
            assert_eq!(minimap_ui_transform.anchor, Anchor::BottomLeft);
            let (local_mouse_position_x, local_mouse_position_y) =
                get_mouse_position_relative_to_minimap(
                    mouse_position,
                    &minimap_ui_transform,
                    &screen_dimensions,
                );

            if is_within_bounding_box(
                (local_mouse_position_x, local_mouse_position_y),
                minimap_ui_transform,
            ) {
                self.was_lmb_originally_pressed_in_minimap_box = is_lmb_down;
            }

            if self.was_lmb_originally_pressed_in_minimap_box {
                let x_percentage = local_mouse_position_x / minimap_ui_transform.width;
                let y_percentage = local_mouse_position_y / minimap_ui_transform.height;

                let map_width = map.pixel_width() as f32;
                let map_height = map.pixel_height() as f32;

                let x = rescale(
                    x_percentage * map_width,
                    0.0..map_width,
                    (-map_width / 2.0)..(map_width / 2.0),
                );
                let y = rescale(
                    y_percentage * map_height,
                    0.0..map_height,
                    (-map_height / 2.0)..(map_height / 2.0),
                );

                let camera_width = 2.0 / active_camera.matrix[(0, 0)];
                let camera_height = -2.0 / active_camera.matrix[(1, 1)];

                camera_transform.translation_mut().x = x - camera_width / 2.0;
                camera_transform.translation_mut().y = y + camera_height / 2.0;
            }
        }
    }
}

fn get_mouse_position_relative_to_minimap(
    mouse_position: (f32, f32),
    minimap_ui_transform: &UiTransform,
    screen_dimensions: &ScreenDimensions,
) -> (f32, f32) {
    assert_eq!(minimap_ui_transform.anchor, Anchor::BottomLeft);
    let (x, y) = mouse_position;

    (x, screen_dimensions.height() - y)
}

/// Rescale an int to 0.0 - 1.0.
fn normalize(value: f32, min: f32, max: f32) -> f32 {
    (value - min) / (max - min)
}

fn rescale(value: f32, old: Range<f32>, new: Range<f32>) -> f32 {
    let percentage = normalize(value, old.start, old.end);

    percentage * (new.end - new.start) + new.start
}

fn is_within_bounding_box(position: (f32, f32), origin: &UiTransform) -> bool {
    let x_min = origin.local_x;
    let x_max = origin.local_x + origin.width;
    let y_min = origin.local_y;
    let y_max = origin.local_y + origin.height;

    position.0 >= x_min && position.0 <= x_max && position.1 >= y_min && position.1 <= y_max
}
