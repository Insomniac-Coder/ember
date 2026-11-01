#$ test: run-pass
#$ rules: CTL-3b, STD-19
#$ profiles: debug, development, release
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
    for x in xs.iter().chain(owns ys.iter()).chain(owns zs.iter()):
        seen.push(owns x)
    println(seen)

    seen = []
    for x in xs.iter().chain(owns ys.iter()):
        if x == 2:
            break
        seen.push(owns x)
    else:
        seen.push(owns -1)
    println(seen)

    seen = []
    for x in xs.iter().chain(owns ys.iter()):
        if x == 20:
            break
        seen.push(owns x)
    else:
        seen.push(owns -1)
    println(seen)

    seen = []
    for x in xs.iter().chain(owns ys.iter()):
        if x % 2 == 0:
            continue
        seen.push(owns x)
    else:
        seen.push(owns -1)
    println(seen)

    turns = 0
    outer: for x in xs.iter().chain(owns ys.iter()):
        for _ in zs.iter():
            turns += 1
            if x == 10:
                break outer
        turns += 1
    println(turns)

    pairs: Array[(int, int)] = []
    for (i, x) in xs.iter().copy_each().enumerate().chain(owns ys.iter().copy_each().enumerate(start=10)):
        pairs.push(owns (i, x))
    println(pairs)

    total = 0
    for k in (0..3).iter().chain(owns (5..=6).iter()).chain(owns (0..10).iter().take(1)):
        total += k
    println(total + 3)

    println(xs.iter().copy_each().chain(owns ys.iter().copy_each()).to_array())

    for x in xs.iter_mut().chain(owns ys.iter_mut()):
        x += 1
    println(f"{xs} {ys}")

    n = 0
    for _ in xs.iter().chain(owns ys.iter()):
        n += 1
    println(n)

    zipped: Array[(int, int)] = []
    for (a, b) in xs.iter().copy_each().take(1).zip(owns ys.iter().copy_each()).chain(owns xs.iter().copy_each().skip(1).zip(owns ys.iter().copy_each().skip(1).chain(owns zs.iter().copy_each()))):
        zipped.push(owns (a - 1, b - 1))
    println(zipped)
