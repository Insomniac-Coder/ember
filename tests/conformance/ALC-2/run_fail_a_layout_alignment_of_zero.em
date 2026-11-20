#$ test: run-fail
#$ rules: ALC-2
#$ panics: `invalid Layout`: an alignment that is not a power of two
## `[ALC-2]` — 0 is no power of two; the constructor checks the arguments however they are named,
## so no other `Layout` exists.

from std.mem import Layout

fn main():
    l = Layout(size=8, align=0)
    println(l.size)
