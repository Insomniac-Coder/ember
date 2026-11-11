#$ test: run-pass
#$ rules: STR-4
#$ stdout: marked
## `[STR-4]` — a struct may have no fields (`struct Marker: pass`).

struct Marker:
    pass

fn name(m: Marker) -> StrView:
    return "marked"

fn main():
    m = Marker()
    println(name(m))
