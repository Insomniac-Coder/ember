from std.fmt import Formatter, FmtError
#$ test: compile-fail
#$ rules: TYP-39, CELL-2
## `[TYP-39]` - a `Cell` shows a copy of its value, as `get` reads it, since it lends its value to nothing: one
## that is not `Copy` and writes its own `Display` is not shown from a `Cell` (`E2040`; ODR-109).

struct S:
    n: int

extend S implements Display:
    fn fmt(ref mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str("s")

fn main():
    c = Cell(S(1))
    println(c)    #$ error[E2040]: `Cell[S]` cannot show its value by `S`'s written `Display`, which would be lent the cell's value
