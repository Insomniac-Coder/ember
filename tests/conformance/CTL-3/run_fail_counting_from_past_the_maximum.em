#$ test: run-fail
#$ rules: CTL-3, TYP-8
#$ panics: integer overflow
#$ stdout: 254
# `[CTL-3]` — `a..` has no end: counting past the type's maximum is an
# overflow (`[TYP-8]`), found as the counter moves past `255`.

for i in 254u8..:
    println(i)
