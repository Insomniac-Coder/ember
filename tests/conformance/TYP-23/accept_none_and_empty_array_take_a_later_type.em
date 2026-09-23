#$ test: run-pass
#$ rules: TYP-23
#$ profiles: debug, release, shipping
#$ stdout: 9 true
#$ 4 6
#$ 9 ann
#$ 0.0
# `[TYP-23]` — `x = None` and `xs = []` leave the type open; a later
# assignment, `push`, or a site expecting a type fixes it.

fn largest(xs: Array[int]) -> Option[int]:
    best = None
    for x in xs:
        if best.is_none() or x > best.unwrap():
            best = x
    return best

fn evens(n: int) -> Array[int]:
    out = []
    for i in range(n):
        if i % 2 == 0:
            out.push(i)
    return out

fn total(xs: Span[float]) -> float:
    return sum(xs)

fn main():
    println(largest([3, 9, 4]).unwrap(), largest([]).is_none())
    println(len(evens(7)), evens(7)[3])
    w = None
    w = 9
    name = None
    name = "ann"
    println(w.unwrap(), name.unwrap())
    fs = []
    println(total(fs))
