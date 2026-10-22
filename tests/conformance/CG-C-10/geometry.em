#$ test: parse-pass
#$ rules: CG-C-10
# The module of the cases beside it: a method and a function calling it, frames they check.
pub struct Grid:
    pub cells: Array[int]

    pub fn at(self, i: int) -> int:
        return self.cells[i]

pub fn probe(g: Grid, i: int) -> int:
    return g.at(i) + g.at(0)
