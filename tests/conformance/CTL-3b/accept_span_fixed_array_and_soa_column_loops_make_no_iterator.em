#$ test: run-pass
#$ rules: CTL-3b
#$ profiles: debug, development, release
#$ assert-c-count: contains("_next(") == 0
#$ stdout: 24 3 10
## `[CTL-3b]` - a `for` over a `Span`, a fixed array `[T; N]` and an `SoA` column (and `enumerate` over a
## view) is an induction-variable loop: no iterator object, no `_next` call per element.

struct P:
    life: int
    mass: int

fn main():
    xs = [1, 2, 3]
    s = xs.as_span()
    t = 0
    for x in s:
        t += x
    for i, x in s.iter().enumerate():
        t += i * x
    a: [int; 4] = [1, 2, 3, 4]
    for x in a:
        t += x
    ps = SoA[P].with_capacity(4)
    ps.push(owns P(1, 2))
    ps.push(owns P(2, 3))
    lives = 0
    for l in ps.life:
        lives += l
    masses = 0
    for m in ps.mass:
        masses += m * 2
    println(t, lives, masses)
