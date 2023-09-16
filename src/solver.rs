use std::fmt::Debug;

use log::info;

use crate::sudoku;

use super::sudoku::Game;

#[derive(Debug)]
pub struct Solver {
    steps: Vec<Box<dyn SolverStep>>,
    current_step: Option<usize>,
}

impl Default for Solver {
    fn default() -> Self {
        Self {
            steps: vec![],
            current_step: None,
        }
    }
}

impl Solver {
    pub fn add_step<S: SolverStep + 'static>(&mut self, step: S) {
        self.steps.push(Box::new(step));
    }

    pub fn count_steps(&self) -> usize {
        self.steps.len()
    }

    pub fn next_step(&mut self, game: Game) -> Game {
        info!(
            "running next step in solver with {} steps",
            self.count_steps()
        );
        if self.count_steps() == 0 {
            return game;
        }

        let next_step = if let Some(current_step) = self.current_step {
            current_step + 1
        } else {
            0
        };

        let next_step = next_step % self.count_steps();
        info!("next step: {}: {:?}", next_step, self.steps[next_step]);

        self.current_step = Some(next_step);
        self.steps[next_step].apply(game)
    }
}

pub trait SolverStep: Debug {
    fn apply(&self, state: Game) -> Game;
}

/// a solver step that reduces the "possible values" of each cell by eliminating every value
/// that is already set in the same unit
#[derive(Debug)]
pub struct GenerateBasicMarkingsStep;

impl SolverStep for GenerateBasicMarkingsStep {
    fn apply(&self, state: Game) -> Game {
        let mut seen_numbers = [0u16; 27];
        /** 27 = 9 rows + 9 cols + 9 groups */
        for row in 0..9 {
            for cell in 0..9 {
                let value = state.get(row + 1, cell + 1);
                if value > 0 {
                    info!("add seen digit: {}: {}", value, 1u16 << (value - 1));
                    let box_id = (row / 3) * 3 + (cell / 3);
                    seen_numbers[row] += 1u16 << (value - 1) as i16;
                    seen_numbers[9 + cell] += 1u16 << (value - 1) as i16;
                    seen_numbers[18 + box_id] += 1u16 << (value - 1) as i16;
                }
            }
        }

        let mut new_game = state;
        for idx in 0..81 {
            let (row, col) = sudoku::Game::cell_index_to_coords(idx);
            let value = state.get(row, col);

            if value == 0 {
                let box_id = ((row - 1) / 3) * 3 + ((col - 1) / 3);
                //TODO: use the existing note, if set, instead of always assuming 0x01FF?
                new_game.set_notes(
                    row,
                    col,
                    0x01FF
                        & !seen_numbers[row - 1]
                        & !seen_numbers[9 + col - 1]
                        & !seen_numbers[18 + box_id],
                )
            }
        }
        new_game
    }
}

/// if the previous steps left only a single candidate in a cell marking, pick that one
#[derive(Debug)]
pub struct NakedSingleStep;

impl SolverStep for NakedSingleStep {
    fn apply(&self, state: Game) -> Game {
        let mut new_game = state;

        for idx in 0..81 {
            let (row, col) = sudoku::Game::cell_index_to_coords(idx);
            let value = state.get(row, col);

            if value == 0 {
                let notes = state.get_notes(row, col);
                match notes {
                    0x0001 => {
                        new_game.set(row, col, 1);
                    }
                    0x0002 => {
                        new_game.set(row, col, 2);
                    }
                    0x0004 => {
                        new_game.set(row, col, 3);
                    }
                    0x0008 => {
                        new_game.set(row, col, 4);
                    }
                    0x0010 => {
                        new_game.set(row, col, 5);
                    }
                    0x0020 => {
                        new_game.set(row, col, 6);
                    }
                    0x0040 => {
                        new_game.set(row, col, 7);
                    }
                    0x0080 => {
                        new_game.set(row, col, 8);
                    }
                    0x0100 => {
                        new_game.set(row, col, 9);
                    }
                    _ => {}
                };
            }
        }

        new_game
    }
}
