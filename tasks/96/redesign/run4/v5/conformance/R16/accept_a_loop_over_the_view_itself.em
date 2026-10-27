#$ test: run-pass
#$ rules: TIER-1
#$ stdout: 6
## design rules: R16
## R16: the function the programmer can write instead of switching checks off: a loop over the view
## itself reads each element once, with no index to check (AN-82's NEW).

fn total(xs: Span[int]) -> int:
    s = 0
    for x in xs:
        s += x
    return s

fn main():
    xs = [1, 2, 3]
    println(total(xs.as_span()))
