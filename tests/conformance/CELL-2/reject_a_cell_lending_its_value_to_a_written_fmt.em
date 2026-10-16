#$ test: compile-fail
#$ rules: CELL-2, TYP-39
# ODR-109 — a value that is not `Copy` and writes its own `Display` or `Debug` cannot be shown from a
# `Cell`: showing it would lend the cell's value (`[CELL-2]`); the help names `RefCell`.

from std.fmt import Formatter, FmtError

struct Name:
    s: String

extend Name implements Display:
    fn fmt(self, mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str(self.s)

extend Name implements Debug:
    fn fmt_debug(self, mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str("N")


fn main():
    n = Cell(Name("x"))
    println(n)    #$ error[E2040]: `Cell[Name]` cannot show its value by `Name`'s written `Display`, which would be lent the cell's value
    println([n])    #$ error[E2040]: `Cell[Name]` cannot show its value by `Name`'s written `Debug`, which would be lent the cell's value
