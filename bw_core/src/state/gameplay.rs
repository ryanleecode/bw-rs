use amethyst::{
    core::Time,
    ecs::Entity,
    prelude::*,
    ui::{UiFinder, UiText},
    utils::fps_counter::FpsCounter,
    SimpleState,
};
use log::info;

#[derive(Default)]
pub struct GameplayState {
    paused: bool,
    fps_display: Option<Entity>,
}

impl SimpleState for GameplayState {
    fn on_start(&mut self, _: StateData<'_, GameData<'_, '_>>) {
        info!("GameplayState started");
    }

    fn on_pause(&mut self, _: StateData<'_, GameData<'_, '_>>) {
        self.paused = true;
    }

    fn on_resume(&mut self, _: StateData<'_, GameData<'_, '_>>) {
        self.paused = false;
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let StateData { world, .. } = data;

        if self.fps_display.is_none() {
            world.exec(|finder: UiFinder<'_>| {
                if let Some(entity) = finder.find("fps") {
                    self.fps_display = Some(entity);
                }
            });
        }

        if !self.paused {
            let mut ui_text = world.write_storage::<UiText>();

            {
                if let Some(fps_display) =
                    self.fps_display.and_then(|entity| ui_text.get_mut(entity))
                {
                    const SAMPLE_SIZE: u64 = 20;
                    if world.read_resource::<Time>().frame_number() % SAMPLE_SIZE == 0 {
                        let fps = world.read_resource::<FpsCounter>().sampled_fps();
                        fps_display.text = format!("FPS: {:.*}", 2, fps);
                    }
                }
            }
        }

        Trans::None
    }
}
