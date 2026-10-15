#$ test: run-pass
#$ rules: GRM-38, EFF-1
#$ stdout: 12 4
# `[GRM-38]` — a generator expression borrows what it iterates and allocates nothing: these
# functions keep `@noalloc` (its adapters take their functions as type parameters).

@noalloc
fn doubled_positives(xs: Span[int]) -> int:
    total = 0
    for v in (x * 2 for x in xs if x > 0):
        total += v
    return total

@noalloc
fn summed(xs: Span[int]) -> int:
    return sum(x for x in xs if x > 1)

fn main():
    values = [1, -2, 2, 3]
    println(doubled_positives(values.as_span()), summed(values.as_span()) - 1)
