mod render;
mod tilemap;

use bevy::{prelude::*, render::render_graph::RenderGraph};
use render::TilemapRenderGraphBuilder;
use tilemap::create_tilemap;

pub mod stage {
    /// The tilemap stage, set to run before `POS T_UPDATE` stage.
    pub const TILEMAP: &str = "tilemap";
}

#[derive(Default)]
pub struct BWTilemapPlugin;

impl Plugin for BWTilemapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_stage_before(
            bevy::app::stage::POST_UPDATE,
            stage::TILEMAP,
            SystemStage::parallel(),
        )
        .add_system_to_stage(stage::TILEMAP, create_tilemap.system());

        let resources = app.resources_mut();
        let mut render_graph = resources
            .get_mut::<RenderGraph>()
            .expect("`RenderGraph` is missing.");
        render_graph.add_tilemap_graph(resources);
    }
}
