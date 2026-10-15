#$ test: run-pass
#$ rules: DET-5
#$ stdout: 3.0
# `[DET-5]` — a `@deterministic` function may call one with the default, strict floating point.

fn fused(a: f64, b: f64, c: f64) -> f64:
    return a * b + c

@deterministic
fn step(x: f64) -> f64:
    return fused(x, 2.0, 1.0)

fn main():
    println(step(1.0))
