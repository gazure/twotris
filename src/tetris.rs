#![allow(unused_variables)]
#![allow(dead_code)]

use bevy::prelude::*;
use rand::{Rng, SeedableRng};
use std::fmt::{Display, Formatter, Result as fmtResult};
use tracing::info;

const GRID_WIDTH: usize = 10;
const GRID_HEIGHT: usize = 16;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum TetrisState {
    #[default]
    InGame,
    GameOver,
}

#[derive(Debug, Component)]
struct GameOver;

#[derive(Debug, Resource)]
struct RandomSource(rand_chacha::ChaCha8Rng);

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

#[derive(Debug, Component)]
struct Grid {
    grid: [[bool; GRID_WIDTH]; GRID_HEIGHT],
}

impl Grid {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, x: usize, y: usize, val: bool) {
        if x >= GRID_WIDTH || y >= GRID_HEIGHT {
            error!(
                "Attempted to set a cell outside of the grid: ({}, {})",
                x, y
            );
            return;
        }
        self.grid[y][x] = val;
    }

    pub fn clear(&mut self) {
        self.grid = [[false; GRID_WIDTH]; GRID_HEIGHT];
    }

    fn set_tetromino_values(&mut self, tetromino: &ControlledTetromino, val: bool) {
        for (y, row) in tetromino.current_structure().iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                if *cell {
                    self.set(tetromino.top_left.0 + x, tetromino.top_left.1 + y, val);
                }
            }
        }
    }

    pub fn set_tetromino(&mut self, tetromino: &ControlledTetromino) {
        self.set_tetromino_values(tetromino, true);
    }

    pub fn unset_tetromino(&mut self, tetromino: &ControlledTetromino) {
        self.set_tetromino_values(tetromino, false);
    }

    pub fn is_tetromino_space_open(&self, tetromino: &ControlledTetromino) -> bool {
        for (y, row) in tetromino.current_structure().iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                if *cell && tetromino.top_left.0 + x >= GRID_WIDTH
                    || tetromino.top_left.1 + y >= GRID_HEIGHT
                    || self.grid[tetromino.top_left.1 + y][tetromino.top_left.0 + x]
                {
                    return false;
                }
            }
        }
        true
    }

    pub fn is_tetromino_blocked_left(&self, tetromino: &ControlledTetromino) -> bool {
        for (y, row) in tetromino.current_structure().iter().enumerate() {
            let left = tetromino.top_left.0;
            if left == 0 || (left > 0 && row[0] && self.grid[tetromino.top_left.1 + y][left - 1]) {
                return true;
            }
        }
        false
    }

    pub fn is_tetromino_blocked_right(&self, tetromino: &ControlledTetromino) -> bool {
        for (y, row) in tetromino.current_structure().iter().enumerate() {
            let right = tetromino.top_left.0 + row.len() - 1;
            if right == GRID_WIDTH - 1
                || (right < GRID_WIDTH - 1
                    && row[row.len() - 1]
                    && self.grid[tetromino.top_left.1 + y][right + 1])
            {
                return true;
            }
        }
        false
    }

    pub fn is_tetromino_at_bottom(&self, tetromino: &ControlledTetromino) -> bool {
        let mut checked_cols = vec![];
        for (y, row) in tetromino.current_structure().iter().enumerate().rev() {
            info!("{}, {:?}", y, row);
            for (x, cell) in row.iter().enumerate() {
                if *cell && !checked_cols.contains(&x) {
                    checked_cols.push(x);
                    info!(
                        "Checking cell at ({}, {})",
                        tetromino.top_left.0 + x,
                        tetromino.top_left.1 + y
                    );
                    if tetromino.top_left.1 + y == GRID_HEIGHT - 1
                        || self.grid[tetromino.top_left.1 + y + 1][tetromino.top_left.0 + x]
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn clear_full_grid_rows(&mut self) -> u32 {
        let mut cleared_rows = 0;
        let mut new_grid = [[false; GRID_WIDTH]; GRID_HEIGHT];
        let mut new_row = GRID_HEIGHT - 1;
        for row in self.grid.iter().rev() {
            if row.iter().all(|&cell| cell) {
                cleared_rows += 1;
            } else {
                new_grid[new_row] = *row;
                new_row = new_row.saturating_sub(1);
            }
        }
        self.grid = new_grid;
        cleared_rows
    }
}

impl Default for Grid {
    fn default() -> Self {
        Grid {
            grid: [[false; GRID_WIDTH]; GRID_HEIGHT],
        }
    }
}

impl Display for Grid {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmtResult {
        for row in self.grid.iter() {
            for cell in row.iter() {
                write!(f, "{}", if *cell { "X" } else { "." })?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

pub enum TetrominoType {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

impl TetrominoType {
    pub fn structure_with_rotations(&self) -> Vec<Vec<Vec<bool>>> {
        match self {
            TetrominoType::I => vec![
                vec![vec![true, true, true, true]],
                vec![vec![true], vec![true], vec![true], vec![true]],
            ],
            TetrominoType::O => vec![vec![vec![true, true], vec![true, true]]],
            TetrominoType::T => vec![
                vec![vec![false, true, false], vec![true, true, true]],
                vec![vec![true, false], vec![true, true], vec![true, false]],
                vec![vec![true, true, true], vec![false, true, false]],
                vec![vec![false, true], vec![true, true], vec![false, true]],
            ],
            TetrominoType::S => vec![
                vec![vec![false, true, true], vec![true, true, false]],
                vec![vec![true, false], vec![true, true], vec![false, true]],
            ],
            TetrominoType::Z => vec![
                vec![vec![true, true, false], vec![false, true, true]],
                vec![vec![false, true], vec![true, true], vec![true, false]],
            ],
            TetrominoType::J => vec![
                vec![vec![true, false, false], vec![true, true, true]],
                vec![vec![true, true], vec![true, false], vec![true, false]],
                vec![vec![true, true, true], vec![false, false, true]],
                vec![vec![false, true], vec![false, true], vec![true, true]],
            ],
            TetrominoType::L => vec![
                vec![vec![false, false, true], vec![true, true, true]],
                vec![vec![true, false], vec![true, false], vec![true, true]],
                vec![vec![true, true, true], vec![true, false, false]],
                vec![vec![true, true], vec![false, true], vec![false, true]],
            ],
        }
    }

    pub fn structure(&self) -> Vec<Vec<bool>> {
        match self {
            TetrominoType::I => vec![vec![true, true, true, true]],
            TetrominoType::O => vec![vec![true, true], vec![true, true]],
            TetrominoType::T => vec![vec![false, true, false], vec![true, true, true]],
            TetrominoType::S => vec![vec![false, true, true], vec![true, true, false]],
            TetrominoType::Z => vec![vec![true, true, false], vec![false, true, true]],
            TetrominoType::J => vec![vec![true, false, false], vec![true, true, true]],
            TetrominoType::L => vec![vec![false, false, true], vec![true, true, true]],
        }
    }

    fn random(rng: &mut RandomSource) -> Self {
        let idx = rng.next(0, 7);
        match idx {
            0 => TetrominoType::I,
            1 => TetrominoType::O,
            2 => TetrominoType::T,
            3 => TetrominoType::S,
            4 => TetrominoType::Z,
            5 => TetrominoType::J,
            6 => TetrominoType::L,
            _ => TetrominoType::O,
        }
    }
}

#[derive(Debug, Component)]
struct ControlledTetromino {
    pub structure: Vec<Vec<Vec<bool>>>,
    pub rotation: usize,
    pub top_left: (usize, usize),
    pub timer: Timer,
}

impl ControlledTetromino {
    pub fn new(tetromino_type: TetrominoType) -> Self {
        ControlledTetromino {
            structure: tetromino_type.structure_with_rotations(),
            rotation: 0,
            top_left: ((GRID_WIDTH / 2) - 1, 0),
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        }
    }

    pub fn current_structure(&self) -> &Vec<Vec<bool>> {
        &self.structure[self.rotation]
    }

    pub fn next_structure(&self) -> &Vec<Vec<bool>> {
        &self.structure[(self.rotation + 1) % self.structure.len()]
    }

    pub fn rotate(&mut self) {
        self.rotation = (self.rotation + 1) % self.structure.len();
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let grid = Grid::default();
    let grid_string = grid.to_string();
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        grid,
        TextBundle::from_section(
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
            left: Val::Px(400.0),
            ..default()
        }),
    ));
}

fn spawn_tetromino(
    mut commands: Commands,
    mut random_source: ResMut<RandomSource>,
    mut grid: Query<(&mut Grid, &mut Text)>,
) {
    let (mut grid, mut text) = grid.single_mut();

    let tetromino = ControlledTetromino::new(TetrominoType::random(&mut random_source));

    info!("Spawning a tetromino");
    grid.set_tetromino(&tetromino);
    text.sections[0].value = grid.to_string();
    commands.spawn((tetromino,));
}

fn handle_input(
    input: Res<ButtonInput<KeyCode>>,
    mut grid: Query<(&mut Grid, &mut Text)>,
    mut tetromino: Query<&mut ControlledTetromino>,
) {
    let (mut grid, mut text) = grid.single_mut();
    let mut tetromino = tetromino.iter_mut().next().unwrap();

    if input.just_pressed(KeyCode::ArrowLeft) && !grid.is_tetromino_blocked_left(&tetromino) {
        info!("Moving tetromino left");
        grid.unset_tetromino(tetromino.as_ref());
        tetromino.top_left.0 -= 1;
        grid.set_tetromino(tetromino.as_ref());
    }

    if input.just_pressed(KeyCode::ArrowRight) && !grid.is_tetromino_blocked_right(&tetromino) {
        info!("Moving tetromino right");
        grid.unset_tetromino(tetromino.as_ref());
        tetromino.top_left.0 += 1;
        grid.set_tetromino(tetromino.as_ref());
    }

    if input.just_pressed(KeyCode::ArrowDown) && !grid.is_tetromino_at_bottom(tetromino.as_ref()) {
        info!("Moving tetromino down");
        grid.unset_tetromino(tetromino.as_ref());
        tetromino.top_left.1 += 1;
        grid.set_tetromino(tetromino.as_ref());
    }

    if input.just_pressed(KeyCode::Space) {
        info!("Rotating tetromino");
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

fn handle_timed_movement(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut random_source: ResMut<RandomSource>,
    mut grid: Query<(&mut Grid, &mut Text)>,
    mut tetromino: Query<(Entity, &mut ControlledTetromino)>,
    mut next_state: ResMut<NextState<TetrisState>>,
) {
    let (mut grid, mut text) = grid.single_mut();
    next_state.set(TetrisState::InGame);
    for (tetromino_id, mut tetromino) in tetromino.iter_mut() {
        tetromino.timer.tick(time.delta());

        if tetromino.timer.finished() {
            if grid.is_tetromino_at_bottom(tetromino.as_ref()) {
                info!("Tetromino at bottom, despawning and spawning a new one");
                grid.clear_full_grid_rows();
                commands.get_entity(tetromino_id).unwrap().despawn();
                let tetromino = ControlledTetromino::new(TetrominoType::random(&mut random_source));
                if grid.is_tetromino_space_open(&tetromino) {
                    grid.set_tetromino(&tetromino);
                    commands.spawn(tetromino);
                } else {
                    next_state.set(TetrisState::GameOver);
                }
            } else {
                info!("Moving tetromino down");
                grid.unset_tetromino(tetromino.as_ref());
                tetromino.top_left.1 += 1;
                grid.set_tetromino(tetromino.as_ref());
            }
            text.sections[0].value = grid.to_string();
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
            top: Val::Px(300.0),
            left: Val::Px(600.0),
            ..default()
        }),
    ));
}

fn reset(
    mut next_state: ResMut<NextState<TetrisState>>,
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    mut grid: Query<(&mut Grid, &mut Text)>,
    gameover: Query<Entity, With<GameOver>>,
) {
    if input.just_pressed(KeyCode::KeyR) {
        next_state.set(TetrisState::InGame);
        for entity_id in gameover.iter() {
            commands.entity(entity_id).despawn();
        }
        let (mut grid, mut text) = grid.single_mut();
        let mut rng = RandomSource::default();
        let tetromino = ControlledTetromino::new(TetrominoType::random(&mut rng));
        commands.remove_resource::<RandomSource>();
        commands.insert_resource(RandomSource::default());
        grid.clear();
        grid.set_tetromino(&tetromino);
        commands.spawn(tetromino);
        text.sections[0].value = grid.to_string();
    }
}

pub struct TetrisPlugin;

impl Plugin for TetrisPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RandomSource::default())
            .init_state::<TetrisState>()
            .add_systems(Startup, (setup, spawn_tetromino).chain())
            .add_systems(
                Update,
                (handle_timed_movement, handle_input).run_if(in_state(TetrisState::InGame)),
            )
            .add_systems(OnEnter(TetrisState::GameOver), (game_over,))
            .add_systems(Update, (reset,).run_if(in_state(TetrisState::GameOver)));
    }
}
