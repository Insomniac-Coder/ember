#$ test: run-pass
#$ rules: BRW-12, FN-1
#$ stdout: 3
## design rules: R2, R4
## R2, R4: a `ref mut` parameter of a view given another view points the caller's view at it, for
## the rest of the program (the caller's `best` now views `ys`; R4 allows only another view argument
## to be put there).

fn pick_longer(ref mut best: Span[int], other: Span[int]):
    if other.len() > best.len():
        best = other

fn main():
    xs = [1]
    ys = [1, 2, 3]
    best = xs.as_span()
    pick_longer(ref mut best, ys.as_span())
    println(best.len())
