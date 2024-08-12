use bevy::{input::common_conditions::input_toggle_active, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy::diagnostic;
use iyes_perf_ui::PerfUiPlugin;

mod tetris;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(tetris::TetrisPlugin)
        .add_plugins(WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Backquote)))
        .add_plugins(diagnostic::FrameTimeDiagnosticsPlugin::default())
        .add_plugins(diagnostic::EntityCountDiagnosticsPlugin::default())
        .add_plugins(diagnostic::SystemInformationDiagnosticsPlugin::default())
        .add_plugins(PerfUiPlugin::default())
        .run();
}
