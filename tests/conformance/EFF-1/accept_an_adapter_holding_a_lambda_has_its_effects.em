#$ test: run-pass
#$ rules: EFF-1, STD-19, CLO-1
#$ profiles: debug, development, release
#$ stdout: 6 4
# ODR-278, D-901, `[EFF-1]` — each lambda has a type of its own, so `map` is instantiated per
# lambda and its call is the lambda's: the `@noalloc` function's adapter runs only its own
# lambda, which allocates nothing, though another `map` of the same shape (`widths`) holds one
# that builds an `Array`. Before, the two lambdas had one type, `fn(ref int) -> int`, so one
# `map` instance ran either, and `doubled` was `E4001` (D-877's precision).

@noalloc
fn doubled(xs: Span[int]) -> int:
    it = xs.iter().map(fn(x: ref int) -> int => x * 2)
    total = 0
    for v in it:
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
