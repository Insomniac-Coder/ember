#$ test: run-pass
#$ rules: LT-1, LT-22
#$ stdout: [1] [2, 3]
# `[LT-1]` — the caller keeps viewed exactly the arguments the called function's code shows the
# result points into: `pick` returns `a`, or itself with `a` in the same place, so only `xs` stays
# viewed and `ys` may grow while `r` is used.

fn pick(a: Span[int], b: Span[int], n: int) -> Span[int]:
    if n == 0:
        return a
    return pick(a, b, n - 1)

fn main():
    xs = [1]
    ys = [2]
    r = pick(xs, ys, 3)
    ys.push(3)
    println(r, ys)
