#$ test: run-pass
#$ rules: BCK-6, BCK-2, BRW-2
#$ profiles: debug, development
#$ stdout: [3, 5, 100, 9] [100, 7]
#$ 4
#$ 1
# `[BCK-6]` — liveness is location-sensitive, so these are accepted: a borrow whose last use
# comes before a later write (`[BRW-2]`); a borrow returned on one branch while the other branch
# writes the borrowed place (`first_even`; `[BCK-2]`: the result's loan is live only on the paths
# that reach its `return`); and a loop that stores a borrow in a local declared outside it on one
# iteration and writes the source on the iterations where it did not.

fn first_even(mut xs: MutSpan[int]) -> ref mut int:
    for i in 0..xs.len():
        if xs[i] % 2 == 0:
            return ref mut xs[i]
    xs[0] = 0
    return ref mut xs[0]

fn last_use_then_write() -> int:
    ys = [1, 2, 3]
    total = 0
    for y in ys:
        total += y
    ys.push(total)
    return ys.len()

fn store_or_write() -> int:
    ys = [1, 2, 3]
    best: Option[ref int] = None
    for i in 0..3:
        if i == 1:
            best = Some(ref ys[0])
        elif best.is_none():
            ys[1] = 5
    match best:
        Some(r):
            return r
        None:
            return 0

fn main():
    values = [3, 5, 8, 9]
    r = first_even(values.as_mut_span())
    r = 100
    odd = [1, 7]
    s = first_even(odd.as_mut_span())
    s = 100
    println(values, odd)
    println(last_use_then_write())
    println(store_or_write())
