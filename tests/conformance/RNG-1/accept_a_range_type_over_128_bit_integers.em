#$ test: run-pass
#$ rules: RNG-1, RNG-3a, TYP-1, TYP-13
#$ stdout:
#$ 170141183460469231731687303715884105000 -5 4
#$ 5 999 7
#$ true false true false
#$ Some(4) None true false
#$ [-5, 4, 170141183460469231731687303715884105000]
#$ true false true
# D-421 — a range type over a 128-bit integer is its representation in C:
# its clamp, its `Option` niche, a struct's or an array's equality and a sort
# go through the runtime's 128-bit helpers, so the program means the same
# where the runtime carries the two halves (MSVC's form, which
# `the_128_bit_programs_run_with_the_halves` builds this program in too).

type Big = i128 in -5 ..= 170141183460469231731687303715884105000
type Wide = u128 in 5 .. 1000

struct P:
    a: Big
    b: int

fn main():
    x: i128 = 170141183460469231731687303715884105727
    a = Big.clamped(x)
    b = Big.clamped(-x)
    c: Big = 4
    println(a, b, c)
    y: u128 = 2
    println(Wide.clamped(y), Wide.clamped(y * 1000), Wide.clamped(y + 5))
    println(c < a, a < c, P(a=c, b=1) == P(a=c, b=1), P(a=a, b=1) == P(a=c, b=1))
    some: Option[Big] = Some(c)
    none: Option[Big] = None
    println(some, none, some == Some(c), none == some)
    xs: Array[Big] = [a, b, c]
    xs.sort()
    println(xs)
    ys: Array[Big] = [b, c, a]
    println(xs == ys, (a, c) == (c, a), (a, c) == (a, c))
