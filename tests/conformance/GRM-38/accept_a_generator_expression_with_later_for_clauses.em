#$ test: run-pass
#$ rules: GRM-38, GRM-27
#$ profiles: debug, development, release
#$ stdout: 20 40 30 60
#$ 1 2 3
#$ b c d
#$ 150
## `[GRM-38]` — a generator expression with a later `for` clause is a value too: a lazy iterator
## whose clauses nest left to right as a comprehension's do, a later one's iterable and the
## conditions reading the names of those before it. It borrows what it iterates and allocates
## nothing, each inner iterator held in place; a name a clause binds may go unread by some of the
## clauses after it.

fn products(xs: Span[int], ys: Span[int]) -> int:
    total = 0
    for p in (x * y for x in xs if x > 1 for y in ys if y > x):
        total += p
    return total

fn main():
    xs = [1, 2, 3]
    ys = [10, 20]
    big = (x * y for x in xs if x > 1 for y in ys if y > x)
    out = String()
    for p in big:
        out.push_str(f"{p} ")
    println(out.trim())
    sums = (a + b + c for a in range(2) for b in range(2) if a <= b for c in range(1, 2))
    out = String()
    for s in sums:
        out.push_str(f"{s} ")
    println(out.trim())
    words = ["ab", "cad"]
    letters = (c for w in words for c in w.chars() if c != 'a')
    out = String()
    for c in letters:
        out.push_str(f"{c} ")
    println(out.trim())
    println(products(xs.as_span(), ys.as_span()))
