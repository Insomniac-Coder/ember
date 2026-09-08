#$ test: run-pass
#$ rules: GRM-16, LEX-15, LEX-15a, CTL-7
#$ stdout: 12
#$ stdout: 18
#$ stdout: 7

## `[GRM-16]` — `return`, `break` and `continue` are expressions of type `!`,
## so a `=>` arm may hold one. Under v0.4 this file did not parse: `=>` took an
## expression and `return` was a statement (errata ERR-008, reversed by `OQ-14`).

enum Shape:
    Circle(r: i32)
    Rect(w: i32, h: i32)
    Empty

fn area(s: Shape) -> i32:
    match s:
        Circle(r) => return 3 * r * r
        Rect(w, h) => return w * h
        Empty => return 0

## `[LEX-15a]` — `type` is a v1 keyword, so an alias parses at item level.
type Count = i32

fn count_to(n: Count) -> Count:
    total: Count = 0
    for i in 0..n:
        if i == 3:
            continue
        if i == 7:
            break
        total = total + i
    return total

## `[LEX-15]` — `let` is a keyword in every position, and a `let` field is
## assignable only in `init`.
struct Fixed:
    let size: i32

fn main():
    println(area(Shape.Rect(3, 4)))
    println(count_to(10))
    f: Fixed = Fixed(7)
    println(f.size)
