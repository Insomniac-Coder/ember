#$ test: run-pass
#$ rules: RNG-5, RNG-2
#$ stdout: -0.25 -3 0.75 1
# D-398 — `-a` of a range value is its representation, and so is `~` of an
# integer one; a construction makes a range value of it again.

type Unit = f64 in 0.0 ..= 1.0
type Level = i32 in 0 ..= 10

fn main():
    a = Unit.clamped(0.25)
    l = Level.clamped(3)
    n: f64 = -a
    m: i32 = -l
    back: f64 = Unit.clamped(1.0 + -a)
    flipped: i32 = Level.clamped(~l + 5)
    println(n, m, back, flipped)
