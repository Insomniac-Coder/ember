#$ test: run-pass
#$ rules: FN-1, LT-1, BRW-12
#$ stdout: 3
## design rules: R4, R6, R2
## R4, verdict 42: a `ref mut` parameter whose value is a view names the caller's view, so `best =
## other` points `main`'s `best` at `b`; after the call `best` may point into any of the call's
## arguments (R6; rules section 3, E3063's example).

fn pick_longer(ref mut best: Span[int], other: Span[int]):
    if other.len() > best.len():
        best = other

fn main():
    a = [1]
    b = [1, 2, 3]
    best = a.as_span()
    pick_longer(ref mut best, b.as_span())
    println(best.len())
