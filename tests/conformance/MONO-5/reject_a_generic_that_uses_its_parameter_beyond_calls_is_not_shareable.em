#$ test: compile-fail
#$ rules: MONO-5, MONO-7
# `[MONO-5]` — a generic is shareable at a parameter `T` when `T` appears in its body only as the
# receiver of calls to methods of `T`'s bounds; `@never_specialize` on one that is not is `E2223`.

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
    y: T = x                        #$ error[E2223]: `twice` cannot share one body
    return y.factor() * 2.0

fn main():
    println(twice(Unit(1.0)))
