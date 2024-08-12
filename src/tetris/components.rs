use bevy::prelude::*;
use std::fmt::{Display, Formatter, Result as fmtResult};
use super::RandomSource;


const GRID_WIDTH: usize = 10;
const GRID_HEIGHT: usize = 16;

#[derive(Debug, Component)]
pub struct GameOver;


#[derive(Debug, Component)]
pub struct Grid {
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
            let mut left = tetromino.top_left.0;
            for (i, v) in row.iter().enumerate() {
                if *v {
                    left = i + tetromino.top_left.0;
                    break;
                }
            }
            if left == 0 || (left > 0 && self.grid[tetromino.top_left.1 + y][left - 1]) {
                return true;
            }
        }
        false
    }

    pub fn is_tetromino_blocked_right(&self, tetromino: &ControlledTetromino) -> bool {
        for (y, row) in tetromino.current_structure().iter().enumerate() {
            let mut right = row.len() - 1;
            for (i, v) in row.iter().enumerate().rev() {
                if *v {
                    right = i;
                    break;
                }
            }
            let right = right + tetromino.top_left.0;
            if right == GRID_WIDTH - 1
                || (right < GRID_WIDTH - 1
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
            for (x, cell) in row.iter().enumerate() {
                if *cell && !checked_cols.contains(&x) {
                    checked_cols.push(x);
                    debug!(
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
pub struct GridTetromino(Entity);

impl GridTetromino {
    pub fn new(grid: Entity) -> Self {
        Self(grid)
    }

    pub fn get(&self) -> Entity {
        self.0
    }
}

#[derive(Debug, Component)]
pub struct ControlledTetromino {
    pub structure: Vec<Vec<Vec<bool>>>,
    pub rotation: usize,
    pub top_left: (usize, usize),
    pub timer: Timer,
}

impl ControlledTetromino {
    pub fn new(rng: &mut RandomSource) -> Self {
        Self::new_with_tetromino_type(TetrominoType::random(rng))
    }

    pub fn new_with_tetromino_type(tetromino_type: TetrominoType) -> Self {
        Self {
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
