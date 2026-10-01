#$ test: run-pass
#$ rules: RNG-1, RNG-3a, TYP-37, STD-15
#$ stdout:
#$ true true
#$ [-0.5, -0.0, 0.0, 0.5]
# D-421 — a value of a range type over `f16` is its representation in C:
# a struct's or a tuple's equality compares it by IEEE equality, so `-0.0`
# equals `0.0`, and a sort orders it by totalOrder, `-0.0` before `0.0`
# (`[TYP-37]`), as for an `f16`. Both compared the bits with C's operators.

type Half = f16 in -1.0 ..= 1.0

struct P:
    a: Half

fn main():
    zero = Half.clamped(0.0)
    minus = Half.clamped(-0.0)
    println(P(a=zero) == P(a=minus), (zero, 1) == (minus, 1))
    xs: Array[Half] = [Half.clamped(0.5), zero, minus, Half.clamped(-0.5)]
    xs.sort()
    println(xs)
