use rustsat::clause;
use rustsat::instances::SatInstance;
use rustsat::solvers::Solve;
use rustsat::solvers::SolverResult::Sat;
use rustsat::types::{Assignment, Lit, TernaryVal};

use rustsat_cadical::CaDiCaL;

use super::SudokuSolver;

#[derive(Default)]
pub struct SatSudokuSolver;

impl SudokuSolver for SatSudokuSolver {
    fn solve(&mut self, puzzle: &[[usize; 9]; 9]) -> Option<[[usize; 9]; 9]> {
        let mut model = SudokuSat::new();
        add_minimal_sudoku_constraints(&mut model);
        add_puzzle_clues(&mut model, puzzle);

        let mut solver = CaDiCaL::default();
        solver.add_cnf(model.instance.clone().into_cnf().0).unwrap();

        match solver.solve().unwrap() {
            Sat => {
                let sol = solver.full_solution().unwrap();
                Some(extract_grid(&model, &sol))
            }
            _ => None,
        }
    }
}

// Internal SAT model and helpers specific to the SAT approach
struct SudokuSat {
    instance: SatInstance,
    literals: Vec<Vec<Vec<Lit>>>, // [row][col][digit-1] -> Lit
}

impl SudokuSat {
    fn new() -> Self {
        let mut instance: SatInstance = SatInstance::new();
        let mut literals: Vec<Vec<Vec<Lit>>> = vec![vec![Vec::new(); 9]; 9];

        for row in 0..9 {
            for col in 0..9 {
                for _digit in 1..=9 {
                    let lit = instance.new_lit();
                    literals[row][col].push(lit);
                }
            }
        }

        SudokuSat { instance, literals }
    }
}

fn add_puzzle_clues(sudoku: &mut SudokuSat, clue: &[[usize; 9]; 9]) {
    for row in 0..9 {
        for col in 0..9 {
            let digit = clue[row][col];
            if digit != 0 {
                set_cell(sudoku, row, col, digit);
            }
        }
    }
}

fn set_cell(sudoku: &mut SudokuSat, row: usize, col: usize, digit: usize) {
    debug_assert!((1..=9).contains(&digit));
    sudoku
        .instance
        .add_unit(sudoku.literals[row][col][digit - 1]);
}

fn add_minimal_sudoku_constraints(sudoku: &mut SudokuSat) {
    let instance = &mut sudoku.instance;
    let literals = &sudoku.literals;

    // Each cell must contain at least one digit
    for row in 0..9 {
        for col in 0..9 {
            let clause = (1..=9).map(|d| literals[row][col][d - 1]).collect();
            instance.add_clause(clause);
        }
    }

    // Each number appears at most once in each row
    for row in 0..9 {
        for digit in 1..=9 {
            for col1 in 0..9 {
                for col2 in (col1 + 1)..9 {
                    let clause = clause!(
                        !literals[row][col1][digit - 1],
                        !literals[row][col2][digit - 1]
                    );
                    instance.add_clause(clause);
                }
            }
        }
    }

    // Each number appears at most once in each column
    for col in 0..9 {
        for digit in 1..=9 {
            for row1 in 0..9 {
                for row2 in (row1 + 1)..9 {
                    let clause = clause!(
                        !literals[row1][col][digit - 1],
                        !literals[row2][col][digit - 1]
                    );
                    instance.add_clause(clause);
                }
            }
        }
    }

    // Each number appears at most once in each 3x3 sub-grid
    for digit in 1..=9 {
        for box_row in 0..3 {
            for box_col in 0..3 {
                let mut cells = Vec::with_capacity(9);
                for r in 0..3 {
                    for c in 0..3 {
                        let row = box_row * 3 + r;
                        let col = box_col * 3 + c;
                        cells.push((row, col));
                    }
                }

                for i in 0..cells.len() {
                    for j in (i + 1)..cells.len() {
                        let (row1, col1) = cells[i];
                        let (row2, col2) = cells[j];
                        let clause = clause!(
                            !literals[row1][col1][digit - 1],
                            !literals[row2][col2][digit - 1]
                        );
                        instance.add_clause(clause);
                    }
                }
            }
        }
    }
}

fn extract_grid(sudoku: &SudokuSat, sol: &Assignment) -> [[usize; 9]; 9] {
    let mut grid = [[0usize; 9]; 9];
    for row in 0..9 {
        for col in 0..9 {
            for digit in 1..=9 {
                let lit = sudoku.literals[row][col][digit - 1];
                if sol[lit.var()] == TernaryVal::True {
                    grid[row][col] = digit;
                    break;
                }
            }
        }
    }
    grid
}
