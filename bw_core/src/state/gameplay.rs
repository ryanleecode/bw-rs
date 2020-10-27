use amethyst::{
    core::transform::Transform,
    prelude::*,
    renderer::{ActiveCamera, Camera},
    SimpleState,
};
use log::info;

pub struct GameplayState;

impl GameplayState {
    pub fn new() -> GameplayState {
        GameplayState
    }
}

impl SimpleState for GameplayState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        info!("GameplayState started");
        let StateData { world, .. } = data;

        initialize_camera(world);
    }
}

fn initialize_camera(world: &mut World) {
    let camera_width = 640.0;
    let camera_height = 500.0;

    let camera = Camera::orthographic(0.0, camera_width, 0.0, camera_height, 0.0, 20.0);
    let mut camera_transform = Transform::default();
    camera_transform.set_translation_xyz(-(camera_width / 2.0), camera_height / 2.0, 10.0);

    let camera_entity = world
        .create_entity()
        .with(camera)
        .with(camera_transform)
        .build();
    world.fetch_mut::<ActiveCamera>().entity = Some(camera_entity);
}
