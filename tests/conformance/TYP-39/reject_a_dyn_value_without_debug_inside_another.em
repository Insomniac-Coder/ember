#$ test: compile-fail
#$ rules: TYP-39, TYP-36
# `[TYP-39]` — a value inside a collection is written by its `Debug`: a `dyn` value whose interfaces
# do not reach `Debug` cannot be, though it is written alone by its `Display`.

from std.fmt import Formatter, FmtError

struct Sq:
    s: int

extend Sq implements Display:
    fn fmt(self, mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str("sq")

fn main():
    shown: Array[Box[dyn Display]] = []
    shown.push(Box(Sq(2)))
    println(shown[0])
    println(shown)    #$ error[E2040]: `Box[dyn std.core.Display]` does not implement `Debug`, which printing a `Array[Box[dyn std.core.Display]]` needs
