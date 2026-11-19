#$ test: run-pass
#$ rules: ATT-1, TYP-9, EFF-6a
#$ stdout: 7
#$ stdout: 7.0
## `[ATT-1]` — 0.9.15's names of two attributes: `@no_alias_checks` (0.9.10's `@static_safe`), a
## function that keeps no check of who else holds its data, and `@fp(allow_fma)` (0.9.10's
## `@fp(contract)`), which lets a multiply and an add fuse (`[TYP-9]`).

@no_alias_checks
fn add(a: int, b: int) -> int:
    return a + b

@fp(allow_fma)
fn mul_add(a: f64, b: f64, c: f64) -> f64:
    return a * b + c

fn main():
    println(add(3, 4))
    println(mul_add(2.0, 3.0, 1.0))
