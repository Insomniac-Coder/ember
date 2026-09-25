#$ test: run-pass
#$ rules: RNG-3a, RNG-3
#$ profiles: debug, release, shipping
#$ stdout: 0.25 0.5 0.75 0.9995
#$ true true
# D-316 — an `f16` range clamps between `f16` endpoints; a half-open one's
# top is the greatest `f16` below its bound. It compared the stored bits.

type Unit = f16 in 0.25 ..= 0.75
type Open = f16 in 0.0 .. 1.0

fn main():
    lo: f16 = Unit.clamped(0.0)
    mid: f16 = Unit.clamped(0.5)
    hi: f16 = Unit.clamped(9.0)
    top: f16 = Open.clamped(5.0)
    println(lo, mid, hi, top)
    println(Unit.checked(0.6).is_ok(), Unit.checked(0.9).is_err())
