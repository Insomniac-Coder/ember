# Clause BRW-13.2: "and if the value is a view, the name now views something else (`rest =
# tail.trim()`, `cur = p`), checked by `[BRW-1]` against its new target."
#$ test: run-pass
#$ rules: BRW-13, BRW-1, TYP-14
#$ stdout: 3
#$ stdout: a b
#$ stdout: 2 5
# `hottest = r` re-points `hottest` (it was `[BRW-1]`'s refused "reference is not re-pointed"). After
# `v = ref b`, `a` is no longer viewed, so it may change.

struct Reading:
    celsius: int

fn main():
    readings = [Reading(celsius=1), Reading(celsius=3)]
    hottest = ref readings[0]
    for r in readings:
        if r.celsius > hottest.celsius:
            hottest = r
    println(hottest.celsius)
    text = String("  a b  ")
    rest: StrView = text
    rest = rest.trim()
    println(rest)
    a = 1
    b = 2
    v = ref a
    v = ref b
    a = 5
    println(v, a)
