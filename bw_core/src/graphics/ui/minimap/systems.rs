use super::MinimapMarker;
use amethyst::{
    assets::{AssetStorage, Handle},
    core::{
        ecs::{Read, ReadStorage, System, WriteStorage},
        Transform,
    },
    renderer::ActiveCamera,
    ui::UiTransform,
};
use bw_assets::map::{self, Map};

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
        (active_camera, minimap_marker, map_handle_opt, map_storage, transforms, mut ui_transforms): Self::SystemData,
    ) {
        let minimap_marker_transform_opt = minimap_marker
            .entity()
            .and_then(|entity| ui_transforms.get_mut(entity));
        let camera_transform_opt = active_camera
            .entity
            .and_then(|entity| transforms.get(entity));
        let map_opt = map_handle_opt
            .as_ref()
            .and_then(|map_handle| map_storage.get(&map_handle.clone()));

        if let (Some(minimap_marker_transform), Some(camera_transform), Some(map)) =
            (minimap_marker_transform_opt, camera_transform_opt, map_opt)
        {
            let map_width = map.dimensions.width as u32;
            let map_height = map.dimensions.height as u32;
            let map_px_width: u32 = map_width * map::MEGATILE_PX_SIDE_LEN;
            let map_px_height: u32 = map_height * map::MEGATILE_PX_SIDE_LEN;

            let x_percentage = normalize(
                camera_transform.translation().x as i32,
                -((map_px_width as i32) / 2),
                (map_px_width / 2) as i32,
            );
            let y_percentage = normalize(
                camera_transform.translation().y as i32,
                -((map_px_height as i32) / 2),
                (map_px_height / 2) as i32,
            );

            let minimap_x = (map_width as f32) * x_percentage;
            let minimap_y = (map_height as f32) * y_percentage;

            minimap_marker_transform.local_x = minimap_x;
            minimap_marker_transform.local_y = minimap_y;
        }
    }
}

/// Rescale a value to 0 - 1.
fn normalize(value: i32, min: i32, max: i32) -> f32 {
    (value - min) as f32 / (max - min) as f32
}
