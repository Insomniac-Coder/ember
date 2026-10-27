#$ test: run-pass
#$ rules: LT-1, BRW-16
#$ stdout: 2
## design rules: R6, R8
## R6: `-> Span[int] from a` narrows the result to `a`, so `y` may grow while `w` is used; the nearest
## allowed program to a result that points into both arguments.

fn head(a: Span[int], b: Span[int]) -> Span[int] from a:
    n = b.len()
    if n > a.len():
        n = a.len()
    return a[..n]

fn main():
    x = [1, 2, 3]
    y = [7, 8]
    w = head(x.as_span(), y.as_span())
    y.push(9)
    println(w.len())
