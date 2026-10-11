#$ test: run-pass
#$ rules: EFF-12
#$ profiles: debug, release, shipping
#$ stdout: 6 2

# `[EFF-12]` — `@static_safe` permits bounds and arithmetic checks: indexing a
# view and adding with overflow checks stay.

@static_safe
fn total(xs: Span[int]) -> int:
    t = 0
    for i in 0..xs.len():
        t = t + xs[i]
    return t

fn main():
    xs: Array[int] = [1, 2, 3]
    println(total(xs), xs[1])
