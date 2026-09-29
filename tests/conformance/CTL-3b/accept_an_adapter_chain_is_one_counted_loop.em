#$ test: run-pass
#$ rules: CTL-3b, STD-19, STD-26
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("_next(") == 0
#$ stdout:
#$ 0:10 1:20 2:30 3:40 4:50 5:60 6:70
#$ 20 40
#$ 10,0,1 20,1,2 30,2,3
#$ (5, 60) (6, 70)
#$ [1, 102, 203, 304]
#$ 1/20 2/30 3/40 4/50 5/60 6/70
#$ 5:10 6:20 7:30 (0, 1) (1, 2) 10+1 20+2
# `[CTL-3b]` — a `for` over `enumerate`, `zip`, `take`, `skip`, `step_by` and
# `copied` composed over a view is one counted loop: the chain's counts are
# worked out before it, each turn computes its element from the loop's
# counter, and no iterator's `next` is called. So is Python's `enumerate(xs)`
# and `zip(xs, ys)` (`[STD-26]`), and a view's iterator part used (it counts
# from where it stands).

fn main():
    line: Array[String] = []
    xs: Array[int] = [10, 20, 30, 40, 50, 60, 70]
    ys: Array[int] = [1, 2, 3]
    for (i, x) in xs.iter().enumerate():
        line.push(f"{i}:{x}")
    println(line.join(" "))
    line.clear()
    for x in xs.iter().skip(1).step_by(2).take(2):
        line.push(f"{x}")
    println(line.join(" "))
    line.clear()
    for (a, (b, c)) in xs.iter().zip(ys.iter().enumerate()):
        line.push(f"{a},{b},{c}")
    println(line.join(" "))
    line.clear()
    for p in xs.iter().copied().enumerate().skip(5):
        line.push(f"{p}")
    println(line.join(" "))
    line.clear()
    zs: Array[int] = [1, 2, 3, 4]
    for (i, z) in zs.iter_mut().enumerate():
        z += i * 100
    println(zs)
    it = xs.iter()
    it.next()
    for (i, x) in it.enumerate(start=1):
        line.push(f"{i}/{x}")
    println(line.join(" "))
    line.clear()
    for i, x in enumerate(xs[0..3], start=5):
        line.push(f"{i}:{x}")
    for p in enumerate(ys[0..2], start=0):
        line.push(f"({p.0}, {p.1})")
    for a, b in zip(xs, ys[0..2]):
        line.push(f"{a}+{b}")
    println(line.join(" "))
    line.clear()
