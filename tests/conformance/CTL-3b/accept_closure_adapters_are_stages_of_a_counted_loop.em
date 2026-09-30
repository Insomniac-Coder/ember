#$ test: run-pass
#$ rules: CTL-3b, STD-19, CLO-14
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("_next(") == 0
#$ stdout: [11, 12, 13, 14, 15, 16, 17, 18]
#$ stdout: [2, 4, 6, 8]
#$ stdout: [0, 1, 3, 4, 6, 7]
#$ stdout: [700, 800]
#$ stdout: [1, 2, 3] ended
#$ stdout: [12, 14, 16]
#$ stdout: [0, 1, 4, 9, 16] [0, 1, 4, 9, 16]
#$ stdout: [-6, -5]
#$ stdout: 116
# ADR-091 — `map`, `filter`, `filter_map`, `take_while`, `skip_while` and
# `inspect` above the counted links of a chain are stages of the loop's turn:
# each callable is evaluated once before the loop and called on each item as
# its adapter's `next` would (a `filter` goes on to the next turn, a
# `take_while` ends the loop and its `else` runs, a `break` skips the `else`).
# No adapter is made and no `next` is called (`[CTL-3b]`).

fn main():
    xs = [1, 2, 3, 4, 5, 6, 7, 8]
    k = 10
    out: Array[int] = []
    for y in xs.iter().map(fn(x: ref int) => x + k):
        out.push(y)
    println(out)
    out = []
    for y in xs.iter().filter(fn(x: ref int) => x % 2 == 0):
        out.push(y)
    println(out)
    out = []
    for (i, y) in xs.iter().enumerate().filter(fn(p: (int, ref int)) => p.0 % 3 == 0):
        out.push(i)
        out.push(y)
    println(out)
    out = []
    for y in xs.iter().filter_map(fn(x: ref int) => Some(x * 100) if x > 6 else None):
        out.push(y)
    println(out)
    out = []
    for y in xs.iter().take_while(fn(x: ref int) => x < 4):
        out.push(y)
    else:
        print(out, "ended")
    println()
    out = []
    for y in xs.iter().skip_while(fn(x: ref int) => x < 6).map(fn(x: ref int) => x * 2):
        out.push(y)
    println(out)
    seen: Array[int] = []
    out = []
    for y in (0..5).map(fn(x: int) => x * x).inspect(fn(v: ref int) => seen.push(v)):
        out.push(y)
    println(seen, out)
    out = []
    for y in xs.iter().rev().skip(2).map(fn(x: ref int) => -x):
        if y == -4:
            break
        out.push(y)
    else:
        print("never")
    println(out)
    total = 0
    for y in xs.iter().map(fn(x: ref int) => x * 3 + 1):
        total += y
    println(total)
