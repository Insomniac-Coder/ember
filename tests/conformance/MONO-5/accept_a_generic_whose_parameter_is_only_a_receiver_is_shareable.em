#$ test: run-pass
#$ rules: MONO-5, MONO-7
#$ stdout: 2.0
# `[MONO-5]` — `x` is used only as the receiver of calls to `Scale`'s methods, so the generic is
# shareable.

interface Scale:
    fn factor() -> f64

struct Unit:
    f: f64

extend Unit implements Scale:
    fn factor() -> f64:
        return this.f

@never_specialize
[T: needs Scale]
fn twice(x: T) -> f64:
    return x.factor() * 2.0

fn main():
    println(twice(Unit(1.0)))
