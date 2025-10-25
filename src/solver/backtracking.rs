use super::SudokuSolver;

#[derive(Default)]
pub struct BacktrackingSudokuSolver;

impl SudokuSolver for BacktrackingSudokuSolver {
    fn solve(&mut self, puzzle: &[[usize; 9]; 9]) -> Option<[[usize; 9]; 9]> {
        let mut grid = *puzzle;
        if solve_grid(&mut grid) {
            Some(grid)
        } else {
            None
        }
    }
}

fn solve_grid(grid: &mut [[usize; 9]; 9]) -> bool {
    if let Some((row, col)) = find_empty(grid) {
        for d in 1..=9 {
            if is_valid(grid, row, col, d) {
                grid[row][col] = d;
                if solve_grid(grid) {
                    return true;
                }
                grid[row][col] = 0;
            }
        }
        false
    } else {
        // no empty cells => solved
        true
    }
}

fn find_empty(grid: &[[usize; 9]; 9]) -> Option<(usize, usize)> {
    for r in 0..9 {
        for c in 0..9 {
            if grid[r][c] == 0 {
                return Some((r, c));
            }
        }
    }
    None
}

/// Check if placing digit d at (row, col) is valid
fn is_valid(grid: &[[usize; 9]; 9], row: usize, col: usize, d: usize) -> bool {
    // row
    for c in 0..9 {
        if grid[row][c] == d {
            return false;
        }
    }
    // col
    for r in 0..9 {
        if grid[r][col] == d {
            return false;
        }
    }
    // box
    let br = (row / 3) * 3;
    let bc = (col / 3) * 3;
    for r in br..br + 3 {
        for c in bc..bc + 3 {
            if grid[r][c] == d {
                return false;
            }
        }
    }
    true
}
