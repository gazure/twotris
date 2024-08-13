use bevy::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use bevy::input::common_conditions::input_toggle_active;
#[cfg(not(target_arch = "wasm32"))]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
#[cfg(not(target_arch = "wasm32"))]
use iyes_perf_ui::PerfUiPlugin;
#[cfg(not(target_arch = "wasm32"))]
use bevy::diagnostic;

mod tetris;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(tetris::TetrisPlugin);

    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Backquote)))
        .add_plugins(diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugins(diagnostic::EntityCountDiagnosticsPlugin)
        .add_plugins(diagnostic::SystemInformationDiagnosticsPlugin)
        .add_plugins(PerfUiPlugin);

    app.run();
}
