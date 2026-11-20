#$ test: run-fail
#$ rules: ALC-2
#$ panics: `invalid Layout`: an alignment that is not a power of two
## `[ALC-2]` — `Layout(size, align)` checks that `align` is a power of two, and panics
## (`invalid Layout`) otherwise.

from std.mem import Layout

fn main():
    l = Layout(3, 3)
    println(l.size)
