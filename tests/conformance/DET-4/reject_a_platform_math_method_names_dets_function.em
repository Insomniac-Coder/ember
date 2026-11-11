#$ test: compile-fail
#$ rules: DET-4, DET-2
#$ error[E4070]: `@deterministic` function `wave` reaches a nondeterministic operation
#$ help: `wave` is `@deterministic`, and `x.sin()` (line 9) is the platform's `sin`, whose last bits differ between C libraries; write `det.sin(x)`
# `[DET-4]` — the same help for the method form: the receiver becomes `det`'s argument.

@deterministic
fn wave(x: f64) -> f64:
    return x.sin()

fn main():
    println(wave(1.0))
