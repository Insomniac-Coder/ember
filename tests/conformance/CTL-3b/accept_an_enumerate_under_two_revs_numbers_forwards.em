#$ test: run-pass
#$ rules: CTL-3b, STD-19
#$ profiles: debug, release
#$ stdout: [(9223372036854775806, 1), (9223372036854775807, 2)]
#$ stdout: [(9223372036854775806, 1)]
# `[CTL-3b]` — a `for` over adapters gives the items std's adapters give and
# fails where they fail. Two `rev`s cancel: `Rev(Rev(e)).next()` is
# `e.next()`, so an `enumerate` under them numbers forwards and panics only
# at an item whose number passes `int.MAX`, never before the first turn
# (D-457).

fn main():
    xs: Array[int] = [1, 2, 3]
    top = int.MAX - 1
    a: Array[(int, int)] = []
    for i, x in xs.iter().copied().enumerate(start=top).rev().rev().take(2):
        a.push((i, x))
    println(a)
    b: Array[(int, int)] = []
    for i, x in xs.iter().copied().enumerate(start=top).rev().skip(1).rev().take(1):
        b.push((i, x))
    println(b)
