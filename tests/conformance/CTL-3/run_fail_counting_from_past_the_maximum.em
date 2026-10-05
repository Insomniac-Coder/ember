#$ test: run-fail
#$ rules: CTL-3, TYP-8
#$ panics: integer overflow
#$ stdout: 254
#$ stdout: 255
# `[CTL-3]` — `a..` has no end: it gives its type's maximum, and the value
# after it is an overflow (`[TYP-8]`), found when the loop asks for it. Until
# D-526 the loop stepped before its turn, so `255` was never given.

for i in 254u8..:
    println(i)
