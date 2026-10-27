#$ test: run-pass
#$ rules: R3, R16
#$ stdout: 7.0
## R3, R16: `@fp(allow_fma)` lets the compiler fuse `a * b + c` into one fused multiply-add (the `@fp` row
## of `V5/vocabulary.md`); the result here is exact either way.

@fp(allow_fma)
fn mix(a: f64, b: f64, c: f64) -> f64:
    return a * b + c

fn main():
    println(mix(2.0, 3.0, 1.0))
