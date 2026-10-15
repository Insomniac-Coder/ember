#$ test: compile-fail
#$ rules: DET-1, DET-2, DET-3
#$ error[E4070]: `@deterministic` function `angle` reaches a nondeterministic operation
# `[DET-1]` — `@deterministic` forbids `Nondet`, and the platform library's `sin` is `Nondet`
# (`[DET-2]`), reached here through a callee (`[DET-3]`).

fn wave(x: f64) -> f64:
    return x.sin()

@deterministic
fn angle(x: f64) -> f64:
    return wave(x) * 2.0

fn main():
    println(angle(0.0))
