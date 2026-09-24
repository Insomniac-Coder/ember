#$ test: run-pass
#$ rules: LT-7, LT-1
#$ stdout: 1
#$ a
#$ [4]
# ODR-024 — a call through a callable value borrows what `[LT-1]` gives a
# function declared with the callable type's parameters: its source
# parameters. A `mut int` is not one, so the caller reads `pos` while the
# token lives; a lambda's `Array` parameter is one, so its result may view it.

fn next_token(mut pos: int, src: str) -> str:
    pos += 1
    return src[0..1]

fn main():
    g: fn(mut int, str) -> str = next_token
    pos = 0
    token = g(pos, "abc")
    println(pos)
    println(token)
    first: fn(Array[int]) -> Span[int] = fn(xs) => xs[..1]
    ys = [4, 5]
    println(first(ys))
