#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::cast_precision_loss)]

#[cfg(not(target_arch = "wasm32"))]
use bevy::diagnostic;
#[cfg(not(target_arch = "wasm32"))]
use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
#[cfg(not(target_arch = "wasm32"))]
use iyes_perf_ui::PerfUiPlugin;

mod tetris;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(tetris::TetrisPlugin);

    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(
        WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Backquote)),
    )
    .add_plugins(diagnostic::FrameTimeDiagnosticsPlugin)
    .add_plugins(diagnostic::EntityCountDiagnosticsPlugin)
    .add_plugins(diagnostic::SystemInformationDiagnosticsPlugin)
    .add_plugins(PerfUiPlugin);

    app.run();
}
