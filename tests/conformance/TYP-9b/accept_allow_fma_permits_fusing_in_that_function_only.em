#$ test: run-pass
#$ rules: TYP-9b, TYP-9, ATT-1
#$ profiles: debug, development, release
#$ assert-c: contains("#pragma STDC FP_CONTRACT ON")
#$ assert-c-count: contains("double em_5fused(double a, double b, double c) {") == 1
#$ assert-c-count: contains("float em_5plain(float a, float b, float c) {") == 1
#$ assert-c-order: "float em_5plain(float a, float b, float c) {" then "its `contract` functions"
#$ assert-c-order: "its `contract` functions" then "double em_5fused(double a, double b, double c) {"
#$ stdout: 7.0
#$ stdout: 0.0
# `[TYP-9b]` — `@fp(allow_fma)` permits fused multiply-add within that function only: the function
# next to it is two roundings: `fused` is built in a unit of its own with contraction on
# (`[CG-C-11]`), `plain` stays in the main unit, which has it off.

@fp(allow_fma)
fn fused(a: f64, b: f64, c: f64) -> f64:
    return a * b + c

fn plain(a: f32, b: f32, c: f32) -> f32:
    return a * b + c

fn main():
    println(fused(2.0, 3.0, 1.0))
    a: f32 = 1.0 + 1.0 / 4096.0
    println(plain(a, a, -(1.0 + 1.0 / 2048.0)))
