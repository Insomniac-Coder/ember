#$ test: run-pass
#$ rules: STR-6, STR-1
#$ stdout: 3 [0, 0, 0]
#$ stdout: 0 []
# `[STR-6]` — a struct that declares `init(self, …)` is built by it: the
# fields take their defaults, then `init` runs with the arguments.

struct Grid:
    width: int = 0
    cells: Array[int] = []

    fn init(mut self, width: int = 0):
        self.width = width
        for _ in range(width):
            self.cells.push(0)

fn main():
    g = Grid(3)
    println(g.width, g.cells)
    e = Grid()
    println(e.width, e.cells)
