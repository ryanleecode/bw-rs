use amethyst::{
    assets::{AssetStorage, Handle},
    core::ecs::{Entity, World},
    prelude::*,
    renderer::Texture,
    ui::{Anchor, UiImage, UiTransform},
};
use bw_assets::map::{self, Map};

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Minimap(Option<Entity>);

impl Minimap {
    pub fn entity(&self) -> &Option<Entity> {
        &self.0
    }
}

pub fn create_minimap(params: (&mut World, &Handle<Map>, &Handle<Texture>)) {
    let (world, map_handle, minimap_texture_handle) = params;

    let (map_width, map_height) = {
        let map_storage = world.read_resource::<AssetStorage<Map>>();
        let map = map_storage
            .get(&map_handle)
            .expect("map is missing")
            .clone();

        (map.dimensions.width, map.dimensions.height)
    };

    let minimap = Minimap(Some(
        world
            .create_entity()
            .with(UiTransform::new(
                String::from("minimap"),
                Anchor::BottomLeft,
                Anchor::BottomLeft,
                0f32,
                0f32,
                0.5f32,
                map_width as f32,
                map_height as f32,
            ))
            .with(UiImage::Texture((*minimap_texture_handle).clone()))
            .build(),
    ));

    world.insert(minimap);
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct MinimapMarker(Option<Entity>);

impl MinimapMarker {
    pub fn entity(&self) -> &Option<Entity> {
        &self.0
    }
}

pub fn create_marker_entity(params: (&mut World, Handle<Texture>)) {
    let (world, square_texture_handle) = params;

    let ui_img = UiImage::NineSlice {
        x_start: 0,
        y_start: 0,
        width: map::MAX_WIDTH,
        height: map::MAX_HEIGHT,

        // the texture has the white border as 1px.
        left_dist: 1,
        right_dist: 1,
        top_dist: 1,
        bottom_dist: 1,
        tex: square_texture_handle.clone(),
        texture_dimensions: [map::MAX_WIDTH, map::MAX_HEIGHT],
    };

    let minimap_marker = MinimapMarker(Some(
        world
            .create_entity()
            .with(UiTransform::new(
                String::from("minimap_marker"),
                Anchor::BottomLeft,
                Anchor::BottomLeft,
                0f32,
                0f32,
                0.6f32,
                24.0,
                18.75,
            ))
            .with(ui_img)
            .build(),
    ));

    world.insert(minimap_marker);
}
