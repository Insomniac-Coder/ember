#$ test: run-pass
#$ rules: OPT-2, CTL-3b
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("_ptr = (") == 7
#$ stdout:
#$ 3 7 11 15 19
#$ 60 [6, 4, 4, 4, 4, 4]
#$ 20 6
#$ 270
# The view an inner loop iterates is made once, before an outer loop that
# cannot change the list it views (its length and buffer), and read through a
# pointer of its own; an outer loop that pushes to the list, or rebinds it,
# makes the view again each turn and sees what changed. Counted where each
# such pointer is set (seven views get one): a loop MSVC runs several turns to
# a pass reads it in each turn (ADR-103).

struct P:
    x: int
    v: int

fn make(n: int) -> Array[int]:
    out: Array[int] = []
    for i in 0..n:
        out.push(i)
    return out

fn scaled(xs: Span[int], k: int) -> int:
    total = 0
    for round in 0..k:
        for x in xs:
            total += x * round
    return total

fn main():
    ps: Array[P] = []
    for i in 0..5:
        ps.push(P(i, i + 1))
    for step in 0..3:
        for p in ps.iter_mut():
            p.x += p.v
    line: Array[String] = []
    for p in ps:
        line.push(f"{p.x}")
    println(line.join(" "))
    xs: Array[int] = [1]
    total = 0
    for r in 0..5:
        for x in xs.iter_mut():
            x += 1
        xs.push(r)
        for x in xs:
            total += x
    println(total, xs)
    ys = make(2)
    seen = 0
    for r in 0..4:
        for y in ys:
            seen += y
        ys = make(r + 3)
    println(seen, ys.len())
    println(scaled(make(10), 4))
