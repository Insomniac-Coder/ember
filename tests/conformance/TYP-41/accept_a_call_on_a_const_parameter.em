#$ test: run-pass
#$ rules: TYP-41, CT-1
#$ stdout: 9 7 10
#$ [1, 2, 3, 4] 16
# `[TYP-41]` rule 2 — in generic code a call that names a `const` parameter is an opaque term:
# `square(N)` in one signature is the type `square(N)` in another, and each instance computes it
# (rule 1): `grid[3]` is `[int; 9]`.

fn square(x: int) -> int:
    return x * x

fn grid[const N: int](fill: int) -> [int; square(N)]:
    out: [int; square(N)] = [fill; square(N)]
    return out

fn last[const N: int](g: [int; square(N)]) -> int:
    return g[square(N) - 1] + N

struct Board[const N: int]:
    cells: [int; square(N)]

fn main():
    g = grid[3](7)
    println(len(g), g[8], last[3](g))
    b = Board[2]([1, 2, 3, 4])
    println(b.cells, len(grid[4](1)))
