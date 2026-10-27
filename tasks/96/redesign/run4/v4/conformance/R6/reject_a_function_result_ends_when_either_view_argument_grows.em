#$ test: compile-fail
#$ rules: R6, R8
#$ help: println(w.len())
## R6: with no `from`, a function's result points into every view argument it could come from, so a
## change that may move either list ends it (R8 (b), E3060).

fn longer(a: Span[int], b: Span[int]) -> Span[int]:
    if a.len() >= b.len():
        return a
    return b

fn main():
    x = [1, 2]
    y = [1, 2, 3]
    w = longer(x.as_span(), y.as_span())
    x.push(9)
    println(w.len())    #$ error[E3060]: `x.push(9)` (line 16) may move the list that `w` points into (`longer` may give back a view of `x` or of `y`, line 15), and `w` is used on line 17; use `w` before line 16: move `println(w.len())` above line 16
