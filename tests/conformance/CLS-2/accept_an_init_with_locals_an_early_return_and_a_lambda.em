#$ test: run-pass
#$ rules: CLS-2
#$ stdout: 9 2 0
#$ stdout: 4 1 7
# `[CLS-2]` — a constructor body is ordinary code: locals, compound writes
# to assigned fields, and a `return` once every field holds a value. A
# lambda's `return` leaves the lambda, not the constructor.

class Grid:
    cells: int
    rows: int
    spare: int = 0

    fn init(width: int):
        bump = fn(v: int) -> int:
            return v + 1
        total = width * width
        this.cells = total
        this.rows = 1
        if width > 2:
            this.rows += 1
            return
        this.spare = bump(6)

fn main():
    g = Grid(3)
    println(g.cells, g.rows, g.spare)
    h = Grid(2)
    println(h.cells, h.rows, h.spare)
