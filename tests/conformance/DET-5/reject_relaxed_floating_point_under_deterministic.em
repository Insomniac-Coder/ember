#$ test: compile-fail
#$ rules: DET-5
#$ error[E4072]: reaches relaxed floating point
# `[DET-5]` — a `@fastmath` or `@fp(contract)` function reached from a `@deterministic` one is
# `E4072`.

@fastmath
fn fused(a: f64, b: f64, c: f64) -> f64:
    return a * b + c

@deterministic
fn step(x: f64) -> f64:
    return fused(x, 2.0, 1.0)

fn main():
    println(step(1.0))
