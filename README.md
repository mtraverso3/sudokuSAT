# Sudoku Solver using SAT

A simple Sudoku solver implemented using a SAT (Boolean Satisfiability Problem) approach. 
This solver encodes the Sudoku constraints into a CNF (Conjunctive Normal Form) formula and uses a SAT solver to find a solution.

## Running
Clone the repository and run:
```bash
cargo run --release
```

## TODO
- [ ] Add support for different Sudoku sizes (e.g., 4x4, 16x16)
- [ ] Implement a more efficient encoding for Sudoku constraints
- [ ] Add a user interface for inputting Sudoku puzzles
