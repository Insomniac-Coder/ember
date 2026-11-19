#$ test: compile-fail
#$ rules: DIA-24, DIA-12
#$ error[E1010]: cannot find `conut` in this scope
#$ help: the nearest name is `count`
#$ help: the nearest name is `width`
#$ help: the nearest name is `Point`
#$ error[E1010]: cannot find `wdth` in this scope
#$ error[E1010]: cannot find `tot` in this scope
#$ not-help: `total`
#$ error[E1010]: cannot find type `point` in this scope
#$ not-help: `point`
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
