#$ test: run-fail
#$ rules: PRF-1, PRF-4, TYP-8
#$ profiles: debug, development, release
#$ panics: overflow
# `[PRF-1]` — integer overflow is checked in every profile, not only while debugging.

fn main():
    xs: Array[int] = [9223372036854775807, 1]
    println(xs[0] + xs[1])
