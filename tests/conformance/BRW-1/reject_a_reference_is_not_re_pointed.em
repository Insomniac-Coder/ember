#$ test: compile-fail
#$ rules: BRW-1, DIA-7
#$ help: a reference is not pointed elsewhere
#$ not-help: `ref mut`
# `[BRW-1]` (D-823) — `hottest = r` on a `ref` local is Python's rebinding of a name; a reference is
# never pointed elsewhere (ADR-010), so the help keeps a copy or an index, and never offers `ref mut`,
# which would write `r` into the element `hottest` refers to.

struct Reading:
    celsius: float

fn main():
    readings = [Reading(celsius=1.0), Reading(celsius=3.0)]
    hottest = ref readings[0]
    for r in readings:
        if r.celsius > hottest.celsius:
            hottest = r    #$ error[E3021]: cannot write through a shared reference
    println(hottest.celsius)
