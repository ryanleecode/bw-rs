use amethyst::{
    core::{ArcThreadPool, Time},
    ecs::{Dispatcher, DispatcherBuilder, Entity},
    input::is_close_requested,
    prelude::*,
    ui::{UiFinder, UiText},
    utils::fps_counter::FpsCounter,
    SimpleState,
};
use log::info;

use crate::graphics::{
    camera::CameraTranslationClampSystem,
    ui::{MinimapMarkerCameraTrackingSystem, MinimapMouseMovementTrackingSystem},
};

#[derive(PartialEq)]
enum MinimapClickState {
    Clicked,
    NotClicked,
}

impl Default for MinimapClickState {
    fn default() -> Self {
        MinimapClickState::NotClicked
    }
}

#[derive(Default)]
pub struct GameplayState<'a, 'b> {
    paused: bool,
    fps_display: Option<Entity>,
    minimap_marker: Option<Entity>,
    minimap: Option<Entity>,
    dispatcher: Option<Dispatcher<'a, 'b>>,
}

impl<'a, 'b> SimpleState for GameplayState<'a, 'b> {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        info!("GameplayState started");

        let StateData { world, .. } = data;

        let mut dispatcher_builder = DispatcherBuilder::new();
        dispatcher_builder.add(
            MinimapMouseMovementTrackingSystem::default().pausable(MinimapClickState::Clicked),
            "minimap_camera_mouse_movement_system",
            &[],
        );
        dispatcher_builder.add(
            CameraTranslationClampSystem::default(),
            "camera_translation_clamp_system",
            &["minimap_camera_mouse_movement_system"],
        );
        dispatcher_builder.add(
            MinimapMarkerCameraTrackingSystem::default(),
            "minimap_camera_tracking_system",
            &["camera_translation_clamp_system"],
        );

        let mut dispatcher = dispatcher_builder
            .with_pool((*world.read_resource::<ArcThreadPool>()).clone())
            .build();
        dispatcher.setup(world);

        self.dispatcher = Some(dispatcher);
    }

    fn on_pause(&mut self, _: StateData<'_, GameData<'_, '_>>) {
        self.paused = true;
    }

    fn on_resume(&mut self, _: StateData<'_, GameData<'_, '_>>) {
        self.paused = false;
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let StateData { world, .. } = data;

        if let Some(dispatcher) = self.dispatcher.as_mut() {
            dispatcher.dispatch(&world);
        }

        if self.fps_display.is_none() {
            world.exec(|finder: UiFinder<'_>| {
                self.fps_display = finder.find("fps");
            });
        }
        if self.minimap.is_none() {
            world.exec(|finder: UiFinder<'_>| {
                self.minimap = finder.find("minimap");
            });
        }
        if self.minimap_marker.is_none() {
            world.exec(|finder: UiFinder<'_>| {
                self.minimap_marker = finder.find("minimap_marker");
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

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        let StateData { world, .. } = data;

        match &event {
            StateEvent::Window(event) => {
                if is_close_requested(&event) {
                    Trans::Quit
                } else {
                    Trans::None
                }
            }
            StateEvent::Ui(ui_event) => {
                if self.minimap == Some(ui_event.target)
                    || (self.minimap_marker) == Some(ui_event.target)
                {
                    match ui_event.event_type {
                        amethyst::ui::UiEventType::ClickStart => {
                            *world.write_resource::<MinimapClickState>() =
                                MinimapClickState::Clicked;
                        }
                        amethyst::ui::UiEventType::ClickStop => {
                            *world.write_resource::<MinimapClickState>() =
                                MinimapClickState::NotClicked;
                        }
                        _ => {}
                    }
                }

                Trans::None
            }
            StateEvent::Input(_) => Trans::None,
        }
    }
}
