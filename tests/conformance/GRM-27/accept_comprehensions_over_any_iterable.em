#$ test: run-pass
#$ rules: GRM-27, CTL-1
#$ stdout: [4, 2, 6] [3, 2, 4] [9, 4] ['a', 'c'] [0, 1, 4] [3, 4, 1, 2, 2, 3]
# `[GRM-27]` — a comprehension's clauses iterate anything a `for` loop does: an adapter chain, a
# generator, a text, `enumerate` and `zip` of anything; clauses nest left to right.

fn main():
    xs = [3, 1, 2]
    a = [x * 2 for x in xs.iter().rev()]
    b = [i + v for i, v in enumerate(xs)]
    c = [x for x in (y * y for y in xs) if x > 1]
    d = [ch for ch in "abc" if ch != 'b']
    e = [p.0 * p.1 for p in zip(xs, 0..3)]
    f = [x + y for x in xs.iter() for y in 0..2]
    println(a, b, c, d, e, f)
