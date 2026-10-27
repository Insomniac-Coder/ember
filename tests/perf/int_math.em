#$ perf: scalar
## `[TST-28]` — integer arithmetic in one loop: checked `+` and `*`, `//` and `%` by constants.

fn main():
    total = 0
    for i in 1..20000001:
        total += (i * 7) % 13 + i // 3 - i % 5
    println(total)
