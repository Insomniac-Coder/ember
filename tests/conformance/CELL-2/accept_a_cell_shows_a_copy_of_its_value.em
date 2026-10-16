#$ test: run-pass
#$ rules: CELL-2, TYP-39
#$ profiles: debug, release
#$ stdout: 20C Temp<20> [Temp<20>, Temp<20>]
# ODR-109 — a `Cell` shows the value it holds (`[TYP-39]`) by a copy, as `get` reads it, when the
# value writes its own `Display` or `Debug`: that code may `set` the cell, and a `Cell` lends its
# value to nothing (`[CELL-2]`). Alone and inside a list alike.

from std.fmt import Formatter, FmtError

@derive(Copy)
struct Temp:
    c: int

extend Temp implements Display:
    fn fmt(self, mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str(f"{self.c}C")

extend Temp implements Debug:
    fn fmt_debug(self, mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str(f"Temp<{self.c}>")

fn main():
    t = Cell(Temp(20))
    println(t, f"{t!r}", [t, t])
