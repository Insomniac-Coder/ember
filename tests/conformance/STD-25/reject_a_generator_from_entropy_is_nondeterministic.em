#$ test: compile-fail
#$ rules: STD-25, DET-2
#$ error[E4070]: `@deterministic` function `roll` reaches a nondeterministic operation
# `[STD-25]` — `Rng.from_entropy()` seeds from the system's generator, so it is `Nondet` (`[DET-2]`)
# and a `@deterministic` function cannot reach it; a seeded generator is the same on every run.

import std.random

@deterministic
fn roll() -> int:
    rng = random.Rng.from_entropy()
    return rng.int_in(1..7)

@deterministic
fn replay() -> int:
    rng = random.Rng.seeded(7)
    return rng.int_in(1..7)

fn main():
    println(roll(), replay())
