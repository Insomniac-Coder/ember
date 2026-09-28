#$ test: run-pass
#$ rules: CTL-3b, STD-19
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_range_count_i64(") == 3
#$ stdout: 22 | [5] | [9223372036854775800, 9223372036854775803, 9223372036854775806]
# `[CTL-3b]` — `(a..b).step_by(k)` in a `for` header is a counted loop: the
# number of values is counted once before it, each value is computed from its
# index, and no step past `b` is ever taken, so a range ending at `int`'s top
# does not overflow: one count for each of the three loops. `k` is checked
# once, before the loop (ODR-089).

fn main():
    xs: Array[int] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    total = 0
    for i in (0..xs.len()).step_by(3):
        total += xs[i]
    once: Array[int] = []
    for i in (5..6).step_by(4):
        once.push(i)
    top: Array[int] = []
    for i in (9223372036854775800..9223372036854775807).step_by(3):
        top.push(i)
    println(total, "|", once, "|", top)
