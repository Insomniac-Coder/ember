#$ test: run-pass
#$ rules: GRM-36, BRW-1
#$ stdout: 30
#$ stdout: 31
## `[GRM-36]` — `ref` binds like unary minus: `ref grid.rows[i]` borrows the
## element `grid.rows[i]`, and `ref mut` the same place mutably.

struct Grid:
    rows: Array[int]

fn main():
    grid = Grid(Array[int]())
    grid.rows.push(10)
    grid.rows.push(30)
    i = 1
    cell: ref int = ref grid.rows[i]
    println(cell)
    slot: ref mut int = ref mut grid.rows[i]
    slot += 1
    println(grid.rows[1])
