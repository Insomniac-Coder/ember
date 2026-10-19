#$ test: compile-fail
#$ rules: SIMD-2, SIMD-5
# `[SIMD-2]` — "`@simd(assert)` makes failure `E4020`", and `[SIMD-5]`:
# "`E4020` names the first clause that fails, and for a remaining check its
# reason (`[EFF-11]`)". Each loop here fails one clause of vectorisable form.

fn gather(xs: Span[int], idx: Span[int], mut out: MutSpan[int]):
    @simd(assert)
    for i in 0..idx.len():  #$ error[E4020]: it reaches memory other than a local or a view at the loop counter plus a constant
        out[i] = xs[idx[i]]

fn float_total(xs: Span[float]) -> float:
    s = 0.0
    @simd(assert)
    for i in 0..xs.len():  #$ error[E4020]: floating-point running total `s`
        s += xs[i]
    return s

fn first_negative(xs: Span[int]) -> int:
    found = -1
    @simd(assert)
    for i in 0..xs.len():  #$ error[E4020]: more than one exit
        if xs[i] < 0:
            found = i
            break
    return found

fn letters(t: str) -> int:
    n = 0
    @simd(assert)
    for c in t.chars():  #$ error[E4020]: its trip count is not known before entry
        if c == 'a':
            n += 1
    return n

# A function value whose function is not known where the loop runs.
fn through(xs: Span[int], mut out: MutSpan[int], f: fn(int) -> int):
    @simd(assert)
    for i in 0..xs.len():  #$ error[E4020]: a call the analysis cannot read through
        out[i] = f(xs[i])

fn divide(xs: Span[int], mut out: MutSpan[int], d: int):
    @simd(assert)
    for i in 0..xs.len():  #$ error[E4020]: by a divisor that is not a constant
        out[i] = xs[i] // d

fn shifted(mut xs: MutSpan[int]):
    @simd(assert)
    for i in 0..xs.len() - 1:  #$ error[E4020]: at more than one offset
        xs[i] = xs[i + 1]

# Each loop adds its turn's value to a running total only on some turns;
# the total's overflow check remains (`not_proven_by_analysis`).
fn positive_total(xs: Span[int]) -> int:
    n = 0
    @simd(assert)
    for i in 0..xs.len():  #$ error[E4020]: not_proven_by_analysis
        if xs[i] > 0:
            n += xs[i]
    return n

fn twice(x: int) -> int:
    return 2 * x

fn main():
    a = [1, 2]
    b = [0, 0]
    fs = [twice]
    through(a, b, fs[0])
