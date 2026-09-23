#$ test: run-pass
#$ rules: LT-7
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ 1
# `[LT-7]`'s example: a callee lends a callback a view of its own local, and a
# callback returns a view of an argument to its caller.

fn with_local(f: fn(Span[int]) -> int) -> int:
    tmp = [41]
    return f(tmp)

fn first_of(a: Span[int], b: Span[int], pick: fn(Span[int], Span[int]) -> Span[int]) -> int:
    return pick(a, b)[0]

fn main():
    println(with_local(fn(s) => s[0] + 1))
    xs = [1, 2]
    ys = [3]
    println(first_of(xs, ys, fn(a, b) => a))
