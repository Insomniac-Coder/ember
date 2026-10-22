#$ test: run-pass
#$ rules: EFF-1, GRM-38, STD-19
#$ profiles: debug, development, release
#$ stdout: 4 0 4
# D-877, `[EFF-1]` — a call through a `plain fn` value (a function, or a lambda capturing nothing)
# runs one of the functions the program uses as values of its shape, so its effects are theirs,
# not every effect: a stored generator expression, an adapter chain the `for` loop does not fuse
# and a later-`for`-clause generator, each holding a lambda that captures nothing, run in a
# `@noalloc` function. Each was `E4001`, "a call through a function value".

@noalloc
fn positives(xs: Span[int]) -> int:
    it = (x for x in xs if x > 0)
    total = 0
    for v in it:
        total += v
    return total

@noalloc
fn doubled_negatives(xs: Span[int]) -> int:
    it = xs.iter().map(fn(x: ref int) -> int => x * 2).filter(fn(x: int) -> bool => x < 0)
    total = 0
    for v in it:
        total += v
    return total + 4

@noalloc
fn pairs(xs: Span[int]) -> int:
    count = 0
    for _ in ((a, b) for a in xs for b in xs if a < b):
        count += 1
    return count + 1

fn main():
    xs = [1, -2, 3]
    println(positives(xs.as_span()), doubled_negatives(xs.as_span()), pairs(xs.as_span()))
