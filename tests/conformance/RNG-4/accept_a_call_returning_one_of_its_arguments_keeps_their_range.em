#$ test: run-pass
#$ rules: RNG-4, TYP-8
#$ profiles: debug, release
#$ stdout:
#$ 100002 19999272958
#$ 4
#$ assert-c-count: contains("ember_ck_") == 0
# `[RNG-4]` — `larger` returns one of its parameters unchanged, so a call to
# it gives one of its arguments: `best` stays within `best`'s and
# `x % 100003`'s ranges, and the running total keeps no check (ADR-101).
# `larger` is called from two places, so no compiler's build inlines it.

fn larger[T: Ord](a: T, b: T) -> T:
    if a > b:
        return a
    return b

fn main():
    best = 0
    total = 0
    for i in 0..200000:
        best = larger(best, (i * 7919) % 100003)
        total = total + best
    println(best, total)
    println(larger(3, 4))
