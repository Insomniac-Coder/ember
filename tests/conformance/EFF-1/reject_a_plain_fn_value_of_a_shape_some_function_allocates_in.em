#$ test: compile-fail
#$ rules: EFF-1, STD-19
# D-877, `[EFF-1]` — a call through a `plain fn` value runs one of the functions the program uses as
# values of its shape (one `int` parameter, an `int` result here), whichever the value holds: one
# such lambda elsewhere builds an `Array`, and `map`'s instance is shared by both, so the `@noalloc`
# function's call may allocate (`E4001`).

@noalloc
fn doubled(xs: Span[int]) -> int:
    it = xs.iter().map(fn(x: ref int) -> int => x * 2)
    total = 0
    for v in it:  #$ error[E4001]: `@noalloc` function `doubled` reaches an allocation
        total += v
    return total

fn widths(xs: Span[int]) -> int:
    it = xs.iter().map(fn(x: ref int) -> int => [x, x].len())
    total = 0
    for v in it:
        total += v
    return total

fn main():
    xs = [1, 2]
    println(doubled(xs.as_span()), widths(xs.as_span()))
