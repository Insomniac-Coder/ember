#$ test: run-fail
#$ rules: STD-23, TYP-8
#$ panics: overflow
# `[STD-23]` — a duration's arithmetic is checked: more seconds than nanoseconds can count panics.

from std.time import Duration

fn main():
    d = Duration.from_secs(10000000000)
    println(d.as_nanos())
