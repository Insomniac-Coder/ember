#$ test: run-fail
#$ rules: TIER-1
#$ stdout: 6
#$ panics: index 3 is out of bounds for a length of 3
## design rules: R16
## R16: `@no_alias_checks` forbids only the run-time alias checks; "No switch turns safety checks off":
## the bounds check still runs (AN-81; today's message).

@no_alias_checks
fn total(xs: Span[int], n: int) -> int:
    s = 0
    for i in 0..n:
        s += xs[i]
    return s

fn main():
    xs = [1, 2, 3]
    println(total(xs.as_span(), 3))
    println(total(xs.as_span(), 4))
