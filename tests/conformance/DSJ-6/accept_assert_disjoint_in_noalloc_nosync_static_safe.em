#$ test: run-pass
#$ rules: DSJ-6, DSJ-1, EFF-12
#$ stdout: apart 5.0
#$ overlap 0.0
# `[DSJ-6]` — `assert_disjoint` is usable in a function that is `@noalloc`,
# `@nosync` and `@static_safe` at once: it establishes a fact rather than
# deferring a check, so none of the three contracts rejects it.

from std.mem import assert_disjoint

@noalloc
@nosync
@static_safe
fn first_sum(a: Span[f32], b: Span[f32]) -> f32:
    match assert_disjoint(a, b):
        Ok((x, y)):
            return x.len() as f32 + y.len() as f32
        Err(_):
            return 0.0

fn main():
    xs: Array[f32] = [1.0, 2.0, 3.0, 4.0, 5.0]
    println("apart", first_sum(xs[0..2], xs[2..5]))
    println("overlap", first_sum(xs[0..3], xs[1..4]))
