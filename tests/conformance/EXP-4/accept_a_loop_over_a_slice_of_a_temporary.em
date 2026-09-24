#$ test: run-pass
#$ rules: EXP-4
#$ profiles: debug, release, shipping
#$ stdout: 3
#$ 4
#$ b
#$ c
#$ 5
# `[EXP-4]` — a temporary made by a `for` iterable lives to the end of the
# loop, including one a slice views.

fn make() -> Array[int]:
    return [1, 2, 3, 4]

fn names() -> Array[String]:
    return ["a", "b", "c"]

fn main():
    for x in make()[2..]:
        println(x)
    for n in names()[1..]:
        println(n)
    println(sum(make()[1..3]))
