#![allow(unused_variables)]
#![allow(dead_code)]

use super::components::{ControlledTetromino, Focus, GameOver, Grid, GridTetromino};
use bevy::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use iyes_perf_ui::prelude::PerfUiCompleteBundle;
use rand::{Rng, SeedableRng};
use tracing::debug;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum TetrisState {
    #[default]
    InGame,
    GameOver,
}

#[derive(Debug, Resource)]
pub struct RandomSource(rand_chacha::ChaCha8Rng);

impl Default for RandomSource {
    fn default() -> Self {
        RandomSource(rand_chacha::ChaCha8Rng::from_entropy())
    }
}

impl RandomSource {
    pub fn next(&mut self, min: u32, max: u32) -> u32 {
        self.0.gen_range(min..max)
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    #[cfg(not(target_arch = "wasm32"))]
    commands.spawn(PerfUiCompleteBundle::default());
}

fn spawn_tetromino(
    mut commands: Commands,
    mut random_source: ResMut<RandomSource>,
    mut grid: Query<(Entity, &mut Grid, &mut Text)>,
) {
    for (entity, mut grid, mut text) in grid.iter_mut() {
        debug!("Spawning a tetromino");
        let tetromino = ControlledTetromino::new(random_source.as_mut());
        grid.set_tetromino(&tetromino);
        text.sections[0].value = grid.to_string();
        commands.spawn((tetromino, GridTetromino::new(entity)));
    }
}

fn swap_focus(
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    focus_grid: Query<Entity, With<Focus>>,
    non_focus_grid: Query<(Entity, &Grid), Without<Focus>>,
) {
    if input.just_pressed(KeyCode::KeyF) {
        debug!("Swapping focus");
        for entity in focus_grid.iter() {
            commands.entity(entity).remove::<Focus>();
        }
        for (non_focus_entity, _) in non_focus_grid.iter() {
            commands.entity(non_focus_entity).insert(Focus);
        }
    }
}

fn handle_input(
    input: Res<ButtonInput<KeyCode>>,
    mut grid: Query<(Entity, &mut Grid, &mut Text), With<Focus>>,
    mut tetromino: Query<(&GridTetromino, &mut ControlledTetromino)>,
) {
    for (entity, mut grid, mut text) in grid.iter_mut() {
        for (grid_owner, mut tetromino) in tetromino.iter_mut() {
            if grid_owner.get() != entity {
                continue;
            }

            if input.just_pressed(KeyCode::ArrowLeft) && !grid.is_tetromino_blocked_left(&tetromino)
            {
                debug!("Moving tetromino left");
                grid.unset_tetromino(tetromino.as_ref());
                tetromino.top_left.0 -= 1;
                grid.set_tetromino(tetromino.as_ref());
            }

            if input.just_pressed(KeyCode::ArrowRight)
                && !grid.is_tetromino_blocked_right(&tetromino)
            {
                debug!("Moving tetromino right");
                grid.unset_tetromino(tetromino.as_ref());
                tetromino.top_left.0 += 1;
                grid.set_tetromino(tetromino.as_ref());
            }

            if input.just_pressed(KeyCode::ArrowDown)
                && !grid.is_tetromino_at_bottom(tetromino.as_ref())
            {
                debug!("Moving tetromino down");
                grid.unset_tetromino(tetromino.as_ref());
                tetromino.top_left.1 += 1;
                grid.set_tetromino(tetromino.as_ref());
            }

            if input.just_pressed(KeyCode::Space) {
                debug!("Rotating tetromino");
                let old_rotation = tetromino.rotation;
                grid.unset_tetromino(tetromino.as_ref());
                tetromino.rotate();
                if !grid.is_tetromino_space_open(&tetromino) {
                    tetromino.rotation = old_rotation;
                }
                grid.set_tetromino(tetromino.as_ref());
            }
            text.sections[0].value = grid.to_string();
        }
    }
}

fn handle_timed_movement(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut random_source: ResMut<RandomSource>,
    mut grid: Query<(Entity, &mut Grid, &mut Text)>,
    mut tetromino: Query<(Entity, &GridTetromino, &mut ControlledTetromino)>,
    mut next_state: ResMut<NextState<TetrisState>>,
) {
    for (entity, mut grid, mut text) in grid.iter_mut() {
        for (tetromino_id, grid_owner, mut tetromino) in tetromino.iter_mut() {
            if grid_owner.get() != entity {
                continue;
            }
            tetromino.timer.tick(time.delta());

            if tetromino.timer.finished() {
                if grid.is_tetromino_at_bottom(tetromino.as_ref()) {
                    debug!("Tetromino at bottom, despawning and spawning a new one");
                    grid.clear_full_grid_rows();
                    commands.entity(tetromino_id).despawn();
                    let tetromino = ControlledTetromino::new(random_source.as_mut());
                    if grid.is_tetromino_space_open(&tetromino) {
                        grid.set_tetromino(&tetromino);
                        commands.spawn((tetromino, GridTetromino::new(entity)));
                    } else {
                        next_state.set(TetrisState::GameOver);
                    }
                } else {
                    debug!("Moving tetromino down");
                    grid.unset_tetromino(tetromino.as_ref());
                    tetromino.top_left.1 += 1;
                    grid.set_tetromino(tetromino.as_ref());
                }
                text.sections[0].value = grid.to_string();
            }
        }
    }
}

fn game_over(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    tetromino: Query<Entity, With<ControlledTetromino>>,
) {
    for entity_id in tetromino.iter() {
        commands.entity(entity_id).despawn();
    }
    commands.spawn((
        GameOver,
        TextBundle::from_section(
            "Game Over".to_string(),
            TextStyle {
                font: asset_server.load("fonts/JetBrainsMono-Bold.ttf"),
                font_size: 72.0,
                color: Color::WHITE,
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(500.0),
            left: Val::Px(600.0),
            ..default()
        }),
    ));
}

fn reset(
    mut next_state: ResMut<NextState<TetrisState>>,
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    gameover: Query<Entity, With<GameOver>>,
) {
    if input.just_pressed(KeyCode::KeyR) {
        next_state.set(TetrisState::InGame);
        for entity_id in gameover.iter() {
            commands.entity(entity_id).despawn();
        }
        commands.remove_resource::<RandomSource>();
        commands.insert_resource(RandomSource::default());
    }
}

fn reset_grid(
    mut grid_text: Query<(&mut Grid, &mut Text)>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    if grid_text.iter().len() == 0 {
        for i in 0..2 {
            let grid = Grid::default();
            let grid_string = grid.to_string();
            let text_bundle = TextBundle::from_section(
                grid_string.to_string(),
                TextStyle {
                    font: asset_server.load("fonts/JetBrainsMono-Bold.ttf"),
                    font_size: 36.0,
                    color: Color::WHITE,
                },
            )
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                left: Val::Px(160.0 + (i as f32 * 400.0)),
                ..default()
            });

            if i == 0 {
                commands.spawn((grid, text_bundle, Focus));
            } else {
                commands.spawn((grid, text_bundle));
            }
        }
    } else {
        for (mut grid, mut text) in grid_text.iter_mut() {
            grid.clear();
            text.sections[0].value = grid.to_string();
        }
    }
}

pub struct TetrisPlugin;

impl Plugin for TetrisPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RandomSource::default())
            .init_state::<TetrisState>()
            .add_systems(Startup, setup)
            .add_systems(
                OnEnter(TetrisState::InGame),
                (reset_grid, spawn_tetromino).chain(),
            )
            .add_systems(
                Update,
                (swap_focus, handle_timed_movement, handle_input)
                    .chain()
                    .run_if(in_state(TetrisState::InGame)),
            )
            .add_systems(OnEnter(TetrisState::GameOver), (game_over,))
            .add_systems(Update, (reset,).run_if(in_state(TetrisState::GameOver)));
    }
}
