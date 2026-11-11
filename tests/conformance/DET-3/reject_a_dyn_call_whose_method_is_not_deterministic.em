#$ test: compile-fail
#$ rules: DET-3, EFF-6
#$ error[E4070]: `@deterministic` function `run`
# `[DET-3]` — a call through a `dyn I` value is admitted under `@deterministic` only when the method
# declares `@deterministic` itself: this one does not, though the implementation below does.

pub interface Scale:
    fn apply(x: f64) -> f64

struct Triple:
    pass

extend Triple implements Scale:
    @deterministic
    fn apply(x: f64) -> f64:
        return x * 3.0

@deterministic
fn run(s: ref dyn Scale, x: f64) -> f64:
    return s.apply(x)

fn main():
    t = Triple()
    println(run(ref t, 2.0))
