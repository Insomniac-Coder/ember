#! module deterministic
#$ test: compile-fail
#$ rules: DET-8
#$ error[E4070]: `@deterministic` function `wave` reaches a nondeterministic operation
#$ error[E4070]: `@deterministic` function `main` reaches a nondeterministic operation
# `[DET-8]` — `#! module deterministic` applies to every function in the module, with no opt-out:
# `main` too, which reaches `wave`.

fn wave(x: f64) -> f64:
    return x.cos()

fn main():
    println(wave(0.0))
