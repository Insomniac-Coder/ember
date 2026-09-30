#$ test: run-pass
#$ rules: CTL-3b, STD-19
#$ profiles: debug, release, shipping
#$ stdout:
#$ [1, 2, 3, 10, 20, 100]
#$ [1]
#$ [1, 2, 3, 10]
#$ [1, 3, -1]
#$ 7
#$ [(0, 1), (1, 2), (2, 3), (10, 10), (11, 20)]
#$ 17
#$ [1, 2, 3, 10, 20]
#$ [2, 3, 4] [11, 21]
#$ 5
#$ [(1, 10), (2, 20), (3, 99)]
# `[CTL-3b]` — a `for` over `a.chain(b)` runs one counted loop for each part
# (ADR-083), and does what `Chain`'s `next` does: every part in order; a
# `break` in one part leaves the rest unrun and skips the `else`, which runs
# only when no part breaks; `continue` goes on with the same part; a labelled
# `break` from an inner loop leaves the whole chain. Parts may be adapted
# (`enumerate`, `take`, `zip`), ranges, copies, or views written through.

fn main():
    xs: Array[int] = [1, 2, 3]
    ys: Array[int] = [10, 20]
    zs: Array[int] = [100]

    seen: Array[int] = []
    for x in xs.iter().chain(ys.iter()).chain(zs.iter()):
        seen.push(x)
    println(seen)

    seen = []
    for x in xs.iter().chain(ys.iter()):
        if x == 2:
            break
        seen.push(x)
    else:
        seen.push(-1)
    println(seen)

    seen = []
    for x in xs.iter().chain(ys.iter()):
        if x == 20:
            break
        seen.push(x)
    else:
        seen.push(-1)
    println(seen)

    seen = []
    for x in xs.iter().chain(ys.iter()):
        if x % 2 == 0:
            continue
        seen.push(x)
    else:
        seen.push(-1)
    println(seen)

    turns = 0
    outer: for x in xs.iter().chain(ys.iter()):
        for _ in zs.iter():
            turns += 1
            if x == 10:
                break outer
        turns += 1
    println(turns)

    pairs: Array[(int, int)] = []
    for (i, x) in xs.iter().copied().enumerate().chain(ys.iter().copied().enumerate(start=10)):
        pairs.push((i, x))
    println(pairs)

    total = 0
    for k in (0..3).iter().chain((5..=6).iter()).chain((0..10).iter().take(1)):
        total += k
    println(total + 3)

    println(xs.iter().copied().chain(ys.iter().copied()).to_array())

    for x in xs.iter_mut().chain(ys.iter_mut()):
        x += 1
    println(f"{xs} {ys}")

    n = 0
    for _ in xs.iter().chain(ys.iter()):
        n += 1
    println(n)

    zipped: Array[(int, int)] = []
    for (a, b) in xs.iter().copied().take(1).zip(ys.iter().copied()).chain(xs.iter().copied().skip(1).zip(ys.iter().copied().skip(1).chain(zs.iter().copied()))):
        zipped.push((a - 1, b - 1))
    println(zipped)
