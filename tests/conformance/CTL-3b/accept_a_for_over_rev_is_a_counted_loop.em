#$ test: run-pass
#$ rules: CTL-3b, STD-19
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("next_back") == 0
#$ stdout: [70, 60, 50, 40, 30, 20, 10]
#$ stdout: [4, 3, 2, 1, 0] [3, 2, 1, 0] [1, 2, 3, 4]
#$ stdout: [(6, 70), (5, 60), (4, 50), (3, 40), (2, 30), (1, 20), (0, 10)]
#$ stdout: [(0, 70), (1, 60), (2, 50), (3, 40), (4, 30), (5, 20), (6, 10)]
#$ stdout: [70, 50, 30] [(30, 3), (20, 2), (10, 1)] [70, 60, 50] [30, 20, 10]
#$ stdout: [7, 4, 3]
# `[CTL-3b]` — `rev` composed over a range or a view is an induction-variable
# loop: turn `k` takes the item `last - k` of what `rev` runs over, so no
# iterator is made and `next_back` is never called (ODR-091). Every position
# of `rev` in a chain, over a mutable view too, and `(0..=n).rev()` on the
# range itself.

fn main():
    xs: Array[int] = [10, 20, 30, 40, 50, 60, 70]
    ys: Array[int] = [1, 2, 3]
    a: Array[int] = []
    for x in xs.iter().rev():
        a.push(x)
    println(a)
    b: Array[int] = []
    for i in (0..5).iter().rev():
        b.push(i)
    c: Array[int] = []
    n = 3
    for i in (0..=n).rev():
        c.push(i)
    d: Array[int] = []
    for i in (1..=4).iter().rev().rev():
        d.push(i)
    println(b, c, d)
    e: Array[(int, int)] = []
    for i, x in xs.iter().enumerate().rev():
        e.push((i, x))
    println(e)
    f: Array[(int, int)] = []
    for i, x in xs.iter().rev().enumerate():
        f.push((i, x))
    println(f)
    g: Array[int] = []
    for x in xs.iter().copied().skip(2).step_by(2).rev():
        g.push(x)
    h: Array[(int, int)] = []
    for p, q in xs.iter().zip(ys.iter()).rev():
        h.push((p, q))
    j: Array[int] = []
    for x in xs.iter().rev().take(3):
        j.push(x)
    k: Array[int] = []
    for x in xs.iter().take(3).rev():
        k.push(x)
    println(g, h, j, k)
    ms: Array[int] = [1, 2, 3]
    turn = 0
    for m in ms.iter_mut().rev():
        turn += 1
        m += turn * (turn - 1)
    println(ms)
