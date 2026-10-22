# The module of ../run_fail_a_backtrace_names_paths_and_places.em: a method and a function that
# call it, each frame of the backtrace that test checks.

pub struct Grid:
    pub cells: Array[int]

    pub fn at(self, i: int) -> int:
        return self.cells[i]

pub fn probe(g: Grid, i: int) -> int:
    return g.at(i) + g.at(0)
