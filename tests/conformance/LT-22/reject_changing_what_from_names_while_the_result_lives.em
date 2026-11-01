#$ test: compile-fail
#$ rules: LT-22, LT-1a, BRW-1
# `[LT-22]` — with `from`, the result points into what `from` names, whatever less the body gives:
# `pick` returns a view of a static list under `from xs`, and its caller keeps `ys` viewed, so a
# change of `ys` while the result is used is refused (`[BRW-1]`).

static NONE: [int; 1] = [0]

fn pick(xs: Span[int]) -> Span[int] from xs:
    return NONE[0..1]

fn main():
    ys = [1, 2]
    r = pick(ys)
    ys[0] = 5    #$ error[E3021]
    println(r, ys)
