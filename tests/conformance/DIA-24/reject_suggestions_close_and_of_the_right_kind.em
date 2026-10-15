#$ test: compile-fail
#$ rules: DIA-24, DIA-12
#$ error[E1010]: cannot find `conut` in this scope
#$ help: did you mean `count`?
#$ error[E1010]: cannot find `wdth` in this scope
#$ help: did you mean `width`?
#$ error[E1010]: cannot find `tot` in this scope
#$ not-help: did you mean `total`?
#$ error[E1010]: cannot find type `point` in this scope
#$ help: did you mean `Point`?
#$ not-help: did you mean `point`?
# `[DIA-24]` — a name is suggested when it is at most 1 apart from the unknown one for names of up to
# 4 characters (`wdth`), at most 2 otherwise (`conut`), and so `total` is not offered for `tot`;
# where a type is expected only types are offered: the local `point` is not.

struct Point:
    x: int
    y: int

fn main():
    count = 3
    width = 4
    total = 5
    point = Point(1, 2)
    println(conut, wdth, tot)
    q: point = point
