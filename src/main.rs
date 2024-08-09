use bevy::prelude::*;

mod tetris;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(tetris::TetrisPlugin)
        .run();
}
