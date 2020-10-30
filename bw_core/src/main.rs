use amethyst::{
    assets::Processor,
    core::transform::TransformBundle,
    input::{InputBundle, StringBindings},
    prelude::*,
    renderer::{
        plugins::{RenderDebugLines, RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
    },
    tiles::RenderTiles2D,
    ui::{RenderUi, UiBundle},
    utils::application_root_dir,
};
use amethyst::{tiles::MortonEncoder2D, utils::fps_counter::FpsCounterBundle};
use bw_assets::{
    map::Map,
    mpq::ArcMPQ,
    tileset::{CV5s, VF4s, VR4s, VX4s, WPEs},
};
use fern::colors::{Color, ColoredLevelConfig};
use graphics::camera::{CameraMovementSystem, CameraRotationSystem};
use ron;
use std::{fs::File, str::FromStr};

mod assets;
mod config;
mod graphics;
mod state;

fn setup_logger(level_filter: log::LevelFilter) -> Result<(), fern::InitError> {
    let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .trace(Color::BrightBlack);

    let colors_level = colors_line.clone().info(Color::Green);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{color_line}[{date}][{target}][{level}{color_line}] {message}\x1B[0m",
                color_line = format_args!(
                    "\x1B[{}m",
                    colors_line.get_color(&record.level()).to_fg_str()
                ),
                date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                target = record.target(),
                level = colors_level.color(record.level()),
                message = message,
            ))
        })
        .level(level_filter)
        .filter(|metadata| metadata.target() != "amethyst_utils::fps_counter")
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}

fn main() -> amethyst::Result<()> {
    use amethyst::error::ResultExt;

    let app_root = application_root_dir()
        .with_context(|_| amethyst::Error::from_string("cannot find application root dir"))?;
    let config_dir = app_root.join("config");

    let bw_config_path = config_dir.join("bw_config.ron");

    let bw_config_f = File::open(bw_config_path)
        .with_context(|_| amethyst::error::format_err!("failed to open config path",))?;
    let bw_config: config::BWConfig = ron::de::from_reader(bw_config_f)?;

    setup_logger(log::LevelFilter::from_str(&bw_config.log_level)?)?;

    let display_config_path = config_dir.join("display.ron");
    let assets_dir = app_root.join("assets");

    let binding_path = config_dir.join("bindings.ron");

    let input_bundle =
        InputBundle::<StringBindings>::new().with_bindings_from_file(binding_path)?;

    let game_data = GameDataBuilder::default()
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?
                        .with_clear([0.0, 0.0, 0.0, 1.0]),
                )
                .with_plugin(RenderDebugLines::default())
                .with_plugin(RenderFlat2D::default())
                .with_plugin(RenderTiles2D::<
                    graphics::tile::AmethystTileBridge,
                    MortonEncoder2D,
                    graphics::tile::ScreenBounds,
                >::default())
                .with_plugin(RenderUi::default()),
        )?
        .with_bundle(FpsCounterBundle::default())?
        .with_bundle(TransformBundle::new())?
        .with_bundle(input_bundle)?
        .with_bundle(UiBundle::<StringBindings>::new())?
        .with(Processor::<Map>::new(), "map_processor", &[])
        .with(Processor::<VX4s>::new(), "vx4s_processor", &[])
        .with(Processor::<VR4s>::new(), "vr4s_processor", &[])
        .with(Processor::<VF4s>::new(), "vf4s_processor", &[])
        .with(Processor::<WPEs>::new(), "wpes_processor", &[])
        .with(Processor::<CV5s>::new(), "cv5s_processor", &[])
        .with(Processor::<ArcMPQ>::new(), "mpq_processor", &[])
        .with(
            CameraMovementSystem,
            "camera_movement_system",
            &["input_system"],
        )
        .with(
            CameraRotationSystem,
            "camera_rotation_system",
            &["input_system"],
        );

    let state = state::MatchLoadingState::new(app_root.join("assets"), bw_config);

    let mut game = Application::new(assets_dir, state, game_data)?;

    game.run();

    Ok(())
}
