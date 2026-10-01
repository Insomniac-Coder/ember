#$ test: run-pass
#$ rules: RNG-3, RNG-10
#$ stdout:
#$ -0.5 -1.0 -0.25 -0.0
#$ -3 true
# D-435 — `[RNG-10]`(a): a constant in range builds a range value, and a
# negated constant is a constant. `-0.5` was a negation the checker derives
# no float fact from (D-392), so it was `E2215`, while `-3` built an integer
# range value through `[RNG-4]`.

type Unit = f64 in -1.0 ..= 1.0
type Half = f16 in -1.0 ..= 1.0
type Signed = i32 in -10 ..= 10

fn main():
    a: Unit = -0.5
    b: Unit = -1.0
    c: Half = -0.25
    d: Unit = -0.0
    println(a, b, c, d)
    e: Signed = -3
    x: f64 = -2.0
    println(e, Unit.checked(x).is_err())
