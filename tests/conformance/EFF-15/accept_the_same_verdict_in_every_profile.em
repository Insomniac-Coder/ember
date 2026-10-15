#$ test: run-pass
#$ rules: EFF-15, PHIL-13
#$ profiles: debug, release, shipping
#$ stdout: 6
# `[EFF-15]` — and every profile accepts this: its bounds and overflow checks are `RuntimeCheck`s,
# which `@nopanic(explicit)` allows, whether or not a profile's code keeps them.

@noalloc
@nopanic(explicit)
fn total(xs: Span[int]) -> int:
    sum = 0
    for i in 0..xs.len():
        sum += xs[i]
    return sum

fn main():
    values = [1, 2, 3]
    println(total(values.as_span()))
