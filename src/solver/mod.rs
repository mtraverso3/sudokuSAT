pub mod sat;

pub trait SudokuSolver {
    fn solve(&mut self, puzzle: &[[usize; 9]; 9]) -> Option<[[usize; 9]; 9]>;
}

pub enum SolverKind {
    Sat,
    // Backtracking,
    // ExactCover,
}

pub enum Solver {
    Sat(sat::SatSudokuSolver),
    // Backtracking(backtracking::BacktrackingSudokuSolver),
    // ExactCover(exact_cover::ExactCoverSudokuSolver),
}

impl SudokuSolver for Solver {
    fn solve(&mut self, puzzle: &[[usize; 9]; 9]) -> Option<[[usize; 9]; 9]> {
        match self {
            Solver::Sat(s) => s.solve(puzzle),
            // Solver::Backtracking(s) => s.solve(puzzle),
            // Solver::ExactCover(s) => s.solve(puzzle),
        }
    }
}

pub fn make_solver(kind: SolverKind) -> Solver {
    match kind {
        SolverKind::Sat => Solver::Sat(sat::SatSudokuSolver::default()),
        // SolverKind::Backtracking => Solver::Backtracking(backtracking::BacktrackingSudokuSolver::default()),
        // SolverKind::ExactCover => Solver::ExactCover(exact_cover::ExactCoverSudokuSolver::default()),
    }
}
