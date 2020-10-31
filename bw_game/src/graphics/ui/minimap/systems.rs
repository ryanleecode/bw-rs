use std::ops::Range;

use super::{Minimap, MinimapMarker};
use amethyst::{
    assets::{AssetStorage, Handle},
    core::{
        ecs::{Read, ReadExpect, ReadStorage, System, WriteStorage},
        Transform,
    },
    ecs::Entities,
    ecs::Join,
    input::{InputHandler, StringBindings},
    renderer::ActiveCamera,
    renderer::Camera,
    ui::Anchor,
    ui::UiTransform,
    window::ScreenDimensions,
};
use bw_assets::map::Map;

#[derive(Default)]

pub struct MinimapMarkerCameraTrackingSystem;

impl<'s> System<'s> for MinimapMarkerCameraTrackingSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Camera>,
        Read<'s, ActiveCamera>,
        Read<'s, AssetStorage<Map>>,
        ReadStorage<'s, Handle<Map>>,
        ReadStorage<'s, Transform>,
        ReadStorage<'s, MinimapMarker>,
        ReadStorage<'s, Minimap>,
        WriteStorage<'s, UiTransform>,
    );

    fn run(
        &mut self,
        (
            entities,
            cameras,
            active_camera,
            map_storage,
            map_handles,
            transforms,
            mimimap_markers,
            minimaps,
            mut ui_transforms,
        ): Self::SystemData,
    ) {
        let minimap_ui_transform_opt = (&minimaps, &ui_transforms)
            .join()
            .next()
            .map(|(_, minimap_ui_transform)| minimap_ui_transform.to_owned());
        let mut minimap_markers_join = (&mimimap_markers, &mut ui_transforms, &map_handles).join();
        let mut camera_join = (&cameras, &transforms).join();

        if let (
            Some(minimap_ui_transform),
            Some((minimap_marker_ui_transform, map)),
            Some((_, camera_transform)),
        ) = (
            minimap_ui_transform_opt,
            minimap_markers_join
                .next()
                .and_then(|(_, minimap_marker_ui_transform, map_handle)| {
                    map_storage
                        .get(map_handle)
                        .map(|map| (minimap_marker_ui_transform, map))
                }),
            active_camera
                .entity
                .and_then(|entity| camera_join.get(entity, &entities))
                .or_else(|| camera_join.next()),
        ) {
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

            let minimap_x = rescale(
                x_percentage,
                0.0..1.0,
                -(minimap_ui_transform.width / 2.0)..(minimap_ui_transform.width / 2.0),
            );
            let minimap_y = rescale(
                y_percentage,
                0.0..1.0,
                -(minimap_ui_transform.height / 2.0)..(minimap_ui_transform.height / 2.0),
            );

            minimap_marker_ui_transform.local_x = minimap_x;
            minimap_marker_ui_transform.local_y = minimap_y;
        }
    }
}
/// System that keeps track of mouse movements on the minimap
///
/// This system does not check whether the player has clicked on the minimap
/// to drag it around. It is the responsibility of the dispatcher to determine
/// whether this system should run.
pub struct MinimapMouseMovementTrackingSystem;

impl Default for MinimapMouseMovementTrackingSystem {
    fn default() -> Self {
        MinimapMouseMovementTrackingSystem
    }
}

impl<'s> System<'s> for MinimapMouseMovementTrackingSystem {
    type SystemData = (
        Read<'s, InputHandler<StringBindings>>,
        ReadStorage<'s, Minimap>,
        ReadStorage<'s, UiTransform>,
        ReadExpect<'s, ScreenDimensions>,
        Read<'s, ActiveCamera>,
        ReadStorage<'s, Camera>,
        WriteStorage<'s, Transform>,
        Read<'s, AssetStorage<Map>>,
        ReadStorage<'s, Handle<Map>>,
        Entities<'s>,
    );

    fn run(
        &mut self,
        (
            input,
            minimaps,
            ui_transforms,
            screen_dimensions,
            active_camera,
            cameras,
            mut transforms,
            map_storage,
            map_handles,
            entities,
        ): Self::SystemData,
    ) {
        let mut minimap_join = (&minimaps, &ui_transforms, &map_handles).join();
        let mut camera_join = (&cameras, &mut transforms).join();

        if let (Some((minimap_ui_transform, map)), Some((camera, camera_transform))) = (
            minimap_join
                .next()
                .and_then(|(_, minimap_ui_transform, map_handle)| {
                    map_storage
                        .get(map_handle)
                        .map(|map| (minimap_ui_transform, map))
                }),
            active_camera
                .entity
                .and_then(|entity| camera_join.get(entity, &entities))
                .or_else(|| camera_join.next()),
        ) {
            let mouse_position_opt = input.mouse_position();

            if let Some(mouse_position) = mouse_position_opt {
                assert_eq!(minimap_ui_transform.anchor, Anchor::Middle);
                let (local_mouse_position_x, local_mouse_position_y) =
                    get_mouse_position_relative_to_minimap(mouse_position, &screen_dimensions);

                let minimap_x_offset =
                    minimap_ui_transform.pixel_x() - (minimap_ui_transform.pixel_width() / 2.0);
                let minimap_y_offset =
                    minimap_ui_transform.pixel_y() - (minimap_ui_transform.pixel_height() / 2.0);

                let x_percentage =
                    (local_mouse_position_x - minimap_x_offset) / minimap_ui_transform.width;
                let y_percentage =
                    (local_mouse_position_y - minimap_y_offset) / minimap_ui_transform.height;

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

                let camera_width = 2.0 / camera.matrix[(0, 0)];
                let camera_height = -2.0 / camera.matrix[(1, 1)];

                camera_transform.translation_mut().x = x - camera_width / 2.0;
                camera_transform.translation_mut().y = y + camera_height / 2.0;
            }
        }
    }
}

fn get_mouse_position_relative_to_minimap(
    mouse_position: (f32, f32),
    screen_dimensions: &ScreenDimensions,
) -> (f32, f32) {
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
/*
fn is_within_bounding_box(position: (f32, f32), origin: &UiTransform) -> bool {
    let x_min = origin.local_x;
    let x_max = origin.local_x + origin.width;
    let y_min = origin.local_y;
    let y_max = origin.local_y + origin.height;

    position.0 >= x_min && position.0 <= x_max && position.1 >= y_min && position.1 <= y_max
}
 */
