#$ test: run-pass
#$ rules: RNG-4, TYP-8
#$ profiles: debug, release, shipping
#$ stdout: 2998
#$ assert-c-count: contains("em_scale") == 0
#$ assert-c-count: contains("ember_ck_") == 0
# `[RNG-4]` — `scale` is called from one place, so it is inlined there and
# the range facts see its argument: `i` is below 1,000, so `x * 3 + 1` cannot
# overflow, `% 7` is of a non-negative number, and the running total of
# values below 7 over 1,000 turns cannot either. No check is left, and no
# `scale` is emitted.

fn scale(x: int) -> int:
    return x * 3 + 1

fn main():
    total = 0
    for i in 0..1000:
        total = total + scale(i) % 7
    println(total)
