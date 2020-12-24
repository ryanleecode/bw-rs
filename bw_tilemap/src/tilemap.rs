use bevy::{
    ecs::{Bundle, Local, Res},
    prelude::*,
    render::{
        mesh::Indices,
        pipeline::{PrimitiveTopology, RenderPipeline},
        render_graph::base::MainPass,
    },
};
use bw_bevy_assets::{Dimensions, TileAtlas, TileAtlasLoadedEvent};

use crate::render::TILEMAP_PIPELINE_HANDLE;

#[derive(Bundle)]
struct TilemapBundle {
    tile_atlas: Handle<TileAtlas>,
    draw: Draw,
    visible: Visible,
    render_pipelines: RenderPipelines,
    main_pass: MainPass,
    mesh: Handle<Mesh>,
    transform: Transform,
    global_transform: GlobalTransform,
}

impl Default for TilemapBundle {
    fn default() -> Self {
        let pipeline = RenderPipeline::new(TILEMAP_PIPELINE_HANDLE.typed());

        TilemapBundle {
            tile_atlas: Default::default(),
            draw: Default::default(),
            visible: Visible {
                is_transparent: false,
                ..Default::default()
            },
            render_pipelines: RenderPipelines::from_pipelines(vec![pipeline]),
            main_pass: MainPass,
            mesh: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
        }
    }
}

pub(crate) fn create_tilemap(
    commands: &mut Commands,
    tile_atlas_loaded_events: Res<Events<TileAtlasLoadedEvent>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut event_reader: Local<EventReader<TileAtlasLoadedEvent>>,
) {
    for tile_atlas_loaded_event in event_reader.iter(&tile_atlas_loaded_events) {
        let tile_atlas_handle = &tile_atlas_loaded_event.handle;
        let map_dimensions = &tile_atlas_loaded_event.map_dimensions;
        let mesh = build_mesh(map_dimensions);

        let mesh_handle = meshes.add(mesh);

        commands.spawn(TilemapBundle {
            tile_atlas: tile_atlas_handle.clone(),
            mesh: mesh_handle.clone(),
            ..Default::default()
        });
    }
}

fn build_mesh(dimensions: &Dimensions) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    let width = dimensions.width as u32;
    let height = dimensions.height as u32;
    let area = width * height;

    let mut vertices = Vec::with_capacity((area as usize) * 4);
    for y in 0..height {
        for x in 0..width {
            let y0 = y as f32 - height as f32 / 2.0;
            let y1 = (y + 1) as f32 - height as f32 / 2.0;
            let x0 = x as f32 - width as f32 / 2.0;
            let x1 = (x + 1) as f32 - width as f32 / 2.0;

            vertices.push([x0, -y0, 0.0]);
            vertices.push([x0, -y1, 0.0]);
            vertices.push([x1, -y1, 0.0]);
            vertices.push([x1, -y0, 0.0]);
        }
    }

    let indices = Indices::U32(
        (0..(width * height) as u32)
            .flat_map(|i| {
                let i = i * 4;
                vec![i + 2, i + 1, i, i, i + 3, i + 2]
            })
            .collect(),
    );

    let mut tile_indices: Vec<u32> = Vec::with_capacity((area as usize) * 4);
    for y in 0..height {
        for x in 0..width {
            tile_indices.extend(&[x + y * width; 4]);
        }
    }

    mesh.set_indices(Some(indices));
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.set_attribute("Vertex_Tile_Index", tile_indices);

    mesh
}
