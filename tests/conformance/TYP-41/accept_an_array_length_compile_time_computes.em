#$ test: run-pass
#$ rules: TYP-41, CT-1
#$ stdout: 6 5 4
#$ stdout: 0 1.0 0
## `[TYP-41]` rule 1 — an array length is any expression compile time
## evaluates over constants: `ROWS * COLS` is as legal as a literal (D-587:
## only a literal or a literal constant was).

const ROWS: int = 2
const COLS: int = 3
const SIDE: int = ROWS + 0

fn main():
    grid: [int; ROWS * COLS] = [0; 6]
    other: [f32; 2 * 3 - 1] = [1.0; 5]
    square: [int; SIDE * SIDE] = [0; SIDE * 2]
    println(len(grid), len(other), len(square))
    println(grid[5], other[4], square[3])
