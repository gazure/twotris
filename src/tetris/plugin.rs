#![allow(unused_variables)]
#![allow(dead_code)]

use super::components::{
    ControlledTetromino, Coordinate, DrawGrid, Focus, GameOver, Grid, GridTetromino,
    RowClearedEvent, Score,
};
use bevy::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use iyes_perf_ui::prelude::PerfUiCompleteBundle;
use rand::{Rng, SeedableRng};
use tracing::debug;

const NON_FOCUS_COLOR: Color = Color::linear_rgba(1.0, 0.1, 0.1, 0.9);
const CELL_SIZE: f32 = 20.0;

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
    commands.spawn((
        Score(0),
        TextBundle::from_section(
            "Score: 0".to_string(),
            TextStyle {
                font: asset_server.load("fonts/JetBrainsMono-Bold.ttf"),
                font_size: 36.0,
                color: Color::WHITE,
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(100.0),
            left: Val::Px(800.0),
            ..default()
        }),
    ));

    let controls = "Left/Right/Down: Move\nSpace: Rotate\nF: Swap Grid";
    commands.spawn((TextBundle::from_section(
        controls.to_string(),
        TextStyle {
            font: asset_server.load("fonts/JetBrainsMono-Bold.ttf"),
            font_size: 24.0,
            color: Color::WHITE,
        },
    )
    .with_style(Style {
        position_type: PositionType::Absolute,
        top: Val::Px(200.0),
        left: Val::Px(800.0),
        ..default()
    }),));
}

fn spawn_tetromino(
    mut commands: Commands,
    mut random_source: ResMut<RandomSource>,
    mut grid_query: Query<(Entity, &mut Grid)>,
    mut draw_grid: EventWriter<DrawGrid>,
) {
    for (entity, mut grid) in &mut grid_query {
        debug!("Spawning a tetromino");
        let tetromino = ControlledTetromino::new(random_source.as_mut());
        grid.set_tetromino(&tetromino);
        commands.spawn((tetromino, GridTetromino::new(entity)));
        draw_grid.send(DrawGrid(entity));
    }
}

fn swap_focus(
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut focus_grid: Query<Entity, With<Focus>>,
    mut non_focus_grid: Query<Entity, (With<Grid>, Without<Focus>)>,
    mut draw_grid: EventWriter<DrawGrid>,
) {
    if input.just_pressed(KeyCode::KeyF) {
        debug!("Swapping focus");
        for entity in &mut focus_grid {
            commands.entity(entity).remove::<Focus>();
            draw_grid.send(DrawGrid(entity));
        }
        for non_focus_entity in &mut non_focus_grid {
            commands.entity(non_focus_entity).insert(Focus);
            draw_grid.send(DrawGrid(non_focus_entity));
        }
    }
}

fn handle_input(
    input: Res<ButtonInput<KeyCode>>,
    mut grid: Query<(Entity, &mut Grid), With<Focus>>,
    mut tetromino: Query<(&GridTetromino, &mut ControlledTetromino)>,
    mut draw_grid: EventWriter<DrawGrid>,
) {
    for (entity, mut grid) in &mut grid {
        for (grid_owner, mut tetromino) in &mut tetromino {
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
            draw_grid.send(DrawGrid(entity));
        }
    }
}

fn handle_timed_movement(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut random_source: ResMut<RandomSource>,
    mut grid: Query<(Entity, &mut Grid)>,
    mut tetromino: Query<(Entity, &GridTetromino, &mut ControlledTetromino)>,
    mut next_state: ResMut<NextState<TetrisState>>,
    mut rows_cleared: EventWriter<RowClearedEvent>,
    mut draw_grid: EventWriter<DrawGrid>,
) {
    for (entity, mut grid) in &mut grid {
        for (tetromino_id, grid_owner, mut tetromino) in &mut tetromino {
            if grid_owner.get() != entity {
                continue;
            }
            tetromino.timer.tick(time.delta());

            if tetromino.timer.finished() {
                if grid.is_tetromino_at_bottom(tetromino.as_ref()) {
                    debug!("Tetromino at bottom, despawning and spawning a new one");
                    rows_cleared.send(RowClearedEvent::new(grid.clear_full_grid_rows()));
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
                draw_grid.send(DrawGrid(entity));
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
            "Game Over\nR: Restart".to_string(),
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
    mut commands: Commands,
    mut grid: Query<(Entity, &mut Grid)>,
    mut score: Query<(&mut Score, &mut Text), Without<Grid>>,
    mut visibile_squares: Query<&mut Visibility, With<Coordinate>>,
    asset_server: Res<AssetServer>,
) {
    if grid.iter().len() == 0 {
        for i in 0..2 {
            let grid = Grid::default();
            let height = grid.height();
            let width = grid.width();
            let grid_string = grid.to_string();
            let transform = Transform::from_xyz(-500.0 + (i as f32 * 400.0), 260.0, 1.0);
            let mut entity = commands.spawn((
                grid,
                SpatialBundle {
                    transform,
                    ..default()
                },
            ));
            if i == 0 {
                entity.insert(Focus);
            }
            let color = if i == 0 {
                Color::srgb(1.0, 0.1, 0.1)
            } else {
                Color::srgb(1.0, 1.0, 1.0)
            };
            entity.with_children(|cb| {
                for i in 0..height {
                    for j in 0..width {
                        cb.spawn((
                            Coordinate(j, i),
                            SpriteBundle {
                                transform: Transform::from_xyz(
                                    j as f32 * CELL_SIZE,
                                    i as f32 * CELL_SIZE * -1.0,
                                    0.0,
                                ),
                                visibility: Visibility::Hidden,
                                sprite: Sprite {
                                    color: Color::srgb(0.0, 0.0, 0.0),
                                    custom_size: Some(Vec2::splat(CELL_SIZE)),
                                    ..default()
                                },
                                ..default()
                            },
                        ))
                        .with_children(|cb| {
                            cb.spawn(SpriteBundle {
                                transform: Transform::from_xyz(0.0, 0.0, 1.0),
                                visibility: Visibility::Inherited,
                                sprite: Sprite {
                                    color,
                                    custom_size: Some(Vec2::splat(CELL_SIZE - 2.0)),
                                    ..default()
                                },
                                ..default()
                            });
                        });
                    }
                }
            });
        }
    } else {
        for (entity, mut grid) in &mut grid {
            grid.clear();
            for mut visibility in &mut visibile_squares {
                *visibility = Visibility::Hidden;
            }
        }
    }

    for (mut score, mut text) in &mut score {
        score.reset();
        text.sections[0].value = format!("Score: {}", score.get());
    }
}

fn draw_grid(
    mut dg_events: EventReader<DrawGrid>,
    grid: Query<(Entity, &Grid, Option<&Focus>)>,
    mut visible_squares: Query<(&mut Visibility, &Coordinate, &Parent)>,
) {
    for event in dg_events.read() {
        for (entity, grid, focus) in &grid {
            if entity != event.0 {
                continue;
            }
            let set_coords: Vec<_> = grid.set_coords_iter().collect();
            for (mut visibility, coord, parent) in &mut visible_squares {
                if parent.get() != entity {
                    continue;
                }
                if set_coords.contains(&coord.tuple()) {
                    *visibility = Visibility::Visible;
                } else {
                    *visibility = Visibility::Hidden;
                }
            }
        }
    }
}

fn update_score(
    mut score: Query<(&mut Score, &mut Text)>,
    mut event: EventReader<RowClearedEvent>,
) {
    for event in event.read() {
        for (mut score, mut text) in &mut score {
            score.add_cleared_rows(event.0);
            text.sections[0].value = format!("Score: {}", score.get());
        }
    }
}

pub struct TetrisPlugin;

impl Plugin for TetrisPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RandomSource::default())
            .init_state::<TetrisState>()
            .add_event::<RowClearedEvent>()
            .add_event::<DrawGrid>()
            .add_systems(Startup, setup)
            .add_systems(
                OnEnter(TetrisState::InGame),
                (reset_grid, spawn_tetromino).chain(),
            )
            .add_systems(
                Update,
                (
                    swap_focus,
                    handle_timed_movement,
                    handle_input,
                    update_score,
                    draw_grid,
                )
                    .run_if(in_state(TetrisState::InGame)),
            )
            .add_systems(OnEnter(TetrisState::GameOver), (game_over,))
            .add_systems(Update, (reset,).run_if(in_state(TetrisState::GameOver)));
    }
}
