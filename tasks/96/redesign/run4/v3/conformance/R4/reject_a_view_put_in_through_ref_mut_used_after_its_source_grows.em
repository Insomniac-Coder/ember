#$ test: compile-fail
#$ rules: R4, R6, R8
#$ help: println(best.len())
## R4, R6: after the call, `best` may point into any of the call's arguments, so a change that may
## move `b`'s elements ends it (R8 (b), E3060).

fn pick_longer(ref mut best: Span[int], other: Span[int]):
    if other.len() > best.len():
        best = other

fn main():
    a = [1]
    b = [1, 2, 3]
    best = a.as_span()
    pick_longer(ref mut best, b.as_span())
    b.push(4)
    println(best.len())    #$ error[E3060]: `b.push(4)` (line 16) may move the list that `best` points into (`pick_longer` may have put a view of `b` there on line 15), and `best` is used on line 17; use `best` before line 16
