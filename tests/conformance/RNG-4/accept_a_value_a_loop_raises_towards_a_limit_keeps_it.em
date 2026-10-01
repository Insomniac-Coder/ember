#$ test: run-pass
#$ rules: RNG-4, TYP-8
#$ profiles: debug, release
#$ stdout: 100002 19999272958
#$ assert-c-count: contains("ember_ck_") == 0
# `[RNG-4]` — a value the loop only raises (`best`) towards a limit the loop
# computes (`x % 100003` is at most 100,002) is widened to the nearest
# constant the loop holds, not to `int`'s end, so it keeps that limit; then the
# running total of it over 200,000 turns is at most 200,000 × 100,003, and
# `total + best` keeps no check (ADR-101).

fn main():
    best = 0
    total = 0
    for i in 0..200000:
        x = (i * 7919) % 100003
        if x > best:
            best = x
        total = total + best
    println(best, total)
