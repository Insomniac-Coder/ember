import grid

struct Grid:
    n: int

fn main():
    println(Grid(n = 1).n, grid.cells())
