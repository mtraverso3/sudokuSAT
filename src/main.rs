use rustsat::clause;
use rustsat::instances::SatInstance;
use rustsat::solvers::{Solve, SolverResult};
use rustsat::types::{Assignment, TernaryVal};

fn main() {
    let mut sudoku_sat = new_sudoku_sat();
    add_minimal_sudoku_constraints(&mut sudoku_sat);

    let puzzle: [[usize; 9]; 9] = [
        [0, 3, 6, 0, 0, 0, 9, 0, 0],
        [1, 0, 0, 5, 3, 0, 2, 0, 0],
        [0, 0, 4, 0, 0, 0, 0, 0, 6],
        [0, 4, 7, 0, 0, 0, 0, 5, 3],
        [0, 0, 0, 0, 0, 8, 0, 6, 9],
        [6, 9, 0, 0, 4, 0, 0, 0, 0],
        [0, 0, 0, 8, 0, 7, 0, 0, 1],
        [0, 0, 2, 0, 0, 0, 0, 0, 4],
        [0, 8, 5, 0, 0, 0, 0, 2, 0],
    ];
    add_puzzle_clues(&mut sudoku_sat, &puzzle);

    // Solving the Sudoku SAT instance using CaDiCaL solver
    let mut solver = rustsat_cadical::CaDiCaL::default();
    solver
        .add_cnf(sudoku_sat.instance.clone().into_cnf().0)
        .unwrap();

    let res = solver.solve().unwrap();

    match res {
        SolverResult::Sat => {
            println!("\nSolution found!");
            let sol = solver.full_solution().unwrap();
            print_solution(&sudoku_sat, &sol);
        }
        SolverResult::Unsat => {
            println!("No solution exists for this Sudoku puzzle.");
        }
        SolverResult::Interrupted => {
            println!("Solver was interrupted.");
        }
    }
}

/// Structure to hold Sudoku instance and its literals
struct SudokuSat {
    instance: SatInstance,
    // mapping from (row, col, digit) to literal
    literals: Vec<Vec<Vec<rustsat::types::Lit>>>,
}

/// Function to create a new SudokuSat instance
fn new_sudoku_sat() -> SudokuSat {
    let mut instance: SatInstance = SatInstance::new();

    let mut literals: Vec<Vec<Vec<rustsat::types::Lit>>> = vec![vec![Vec::new(); 9]; 9];
    for row in 0..9 {
        for col in 0..9 {
            for digit in 1..=9 {
                let lit = instance.new_lit();
                literals[row][col].push(lit);
            }
        }
    }

    SudokuSat { instance, literals }
}

/// Add clues from the given Sudoku puzzle to the SAT instance
fn add_puzzle_clues(sudoku: &mut SudokuSat, clue: &[[usize; 9]; 9]) {
    for row in 0..9 {
        for col in 0..9 {
            let digit = clue[row][col];
            if digit != 0 {
                // 0 represents an empty cell
                set_cell(sudoku, row, col, digit);
            }
        }
    }
}

/// Helper function to set a specific cell to a digit (for adding puzzle clues)
#[allow(dead_code)]
fn set_cell(sudoku: &mut SudokuSat, row: usize, col: usize, digit: usize) {
    assert!(digit >= 1 && digit <= 9);
    sudoku
        .instance
        .add_unit(sudoku.literals[row][col][digit - 1]);
}

/// Add minimal Sudoku constraints to the SAT instance
fn add_minimal_sudoku_constraints(sudoku: &mut SudokuSat) {
    let instance = &mut sudoku.instance;
    let literals = &sudoku.literals;

    // Minimum constraints for Sudoku (https://sat.inesc-id.pt/~ines/publications/aimath06.pdf)

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
                // Get all cells in this 3x3 box
                let mut cells = Vec::new();
                for r in 0..3 {
                    for c in 0..3 {
                        let row = box_row * 3 + r;
                        let col = box_col * 3 + c;
                        cells.push((row, col));
                    }
                }

                // Each pair of cells in the box cannot both contain the same digit
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

/// Extended Sudoku constraints, for performance improvement
fn add_extended_sudoku_constraints(sudoku: &mut SudokuSat) {
    let instance = &mut sudoku.instance;
    let literals = &sudoku.literals;

    // TODO: Extended Sudoku constraints
}

/// Helper function to print the solved Sudoku grid
fn print_solution(sudoku: &SudokuSat, sol: &Assignment) {
    println!("\nSolved Sudoku:");
    for row in 0..9 {
        if row % 3 == 0 && row != 0 {
            println!("------+-------+------");
        }
        for col in 0..9 {
            if col % 3 == 0 && col != 0 {
                print!("| ");
            }
            for digit in 1..=9 {
                let lit = sudoku.literals[row][col][digit - 1];
                if sol[lit.var()] == TernaryVal::True {
                    print!("{} ", digit);
                    break;
                }
            }
        }
        println!();
    }
}
