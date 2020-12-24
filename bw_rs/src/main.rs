use std::path::PathBuf;

use bevy::{prelude::*, reflect::TypeUuid};
use bw_bevy_assets::LoadMapEvent;
use bw_tilemap::BWTilemapPlugin;
use serde::Deserialize;

mod plugin;

fn main() {
    App::build()
        .add_resource(WindowDescriptor {
            title: "BW-RS".to_string(),
            width: 640.0,
            height: 500.0,
            vsync: true,
            resizable: true,
            ..Default::default()
        })
        .add_plugins(plugin::DefaultPlugins::default())
        .add_asset::<BWSettings>()
        .add_plugin(BWTilemapPlugin::default())
        .add_startup_system(setup.system())
        .run()
}

#[derive(Deserialize, TypeUuid)]
#[uuid = "89983683-4626-4841-b0cc-7af0960fa640"]
pub struct BWSettings {
    log_level: String,
    map: String,
}

fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut events: ResMut<Events<LoadMapEvent>>,
) {
    commands.spawn(Camera2dBundle::default());

    events.send(LoadMapEvent {
        path: PathBuf::from("maps/(2)Destination.scx"),
    });

    asset_server.watch_for_changes().unwrap();
}
