use amethyst::{
    assets::{AssetStorage, Handle},
    ecs::{Component, NullStorage, World},
    prelude::*,
    renderer::Texture,
    ui::UiFinder,
    ui::{UiImage, UiTransform},
};
use bw_assets::map::Map;

#[derive(Default)]
pub struct MinimapMarker;

impl MinimapMarker {
    pub fn attach(world: &mut World, map_handle: &Handle<Map>) {
        if let Some(minimap_marker) =
            world.exec(|finder: UiFinder<'_>| finder.find("minimap_marker"))
        {
            world
                .write_component::<MinimapMarker>()
                .insert(minimap_marker, MinimapMarker::default())
                .expect("minimap marker component could not be attached");
            world
                .write_component::<Handle<Map>>()
                .insert(minimap_marker, map_handle.clone())
                .expect("failed to add map handle to minimap marker");
        }
    }
}

impl Component for MinimapMarker {
    type Storage = NullStorage<MinimapMarker>;
}

#[derive(Default)]
pub struct Minimap;

impl Minimap {
    pub fn attach(params: (&mut World, &Handle<Map>, &Handle<Texture>)) {
        let (world, map_handle, minimap_texture_handle) = params;

        if let (Some(minimap_container), Some(minimap)) = (
            world.exec(|finder: UiFinder<'_>| finder.find("minimap_container")),
            world.exec(|finder: UiFinder<'_>| finder.find("minimap")),
        ) {
            let (map_width, map_height) = {
                let map_storage = world.read_resource::<AssetStorage<Map>>();
                let map = map_storage.get(&map_handle).expect("map is missing");

                (map.dimensions.width, map.dimensions.height)
            };

            let mut ui_transform = world.write_component::<UiTransform>();
            if let (Some((container_width, container_height)), Some(minimap_ui_transform)) = (
                ui_transform
                    .get(minimap_container)
                    .map(|minimap_container_ui_transform| {
                        (
                            minimap_container_ui_transform.width,
                            minimap_container_ui_transform.height,
                        )
                    }),
                ui_transform.get_mut(minimap),
            ) {
                minimap_ui_transform.width =
                    container_width.min(container_width * (map_width as f32 / map_height as f32));
                minimap_ui_transform.height =
                    container_height.min(container_height * (map_height as f32 / map_width as f32));
            }

            world
                .write_component::<Handle<Map>>()
                .insert(minimap, map_handle.clone())
                .expect("failed to add map handle to minimap marker");

            world
                .write_component::<UiImage>()
                .insert(minimap, UiImage::Texture((*minimap_texture_handle).clone()))
                .expect("failed to add minimap texture");

            world
                .write_component::<Minimap>()
                .insert(minimap, Minimap::default())
                .expect("failed to add minimap component");
        }
    }
}

impl Component for Minimap {
    type Storage = NullStorage<Minimap>;
}
