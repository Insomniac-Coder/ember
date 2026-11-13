# Clause SPN-1.2: "and to `MutSpan[T]` where one is expected (a `ref mut` parameter, a `ref mut`
# name, or a `MutSpan`-typed field made with `ref mut`, `[BRW-12]`)"
#$ test: run-pass
#$ rules: SPN-1, SPN-3, FN-1a, FN-2a, BRW-12
#$ stdout: [9, 9, 9]
#$ stdout: [7, 9, 9]
#$ stdout: [7, 5, 9]
# The three places that expect a `MutSpan`, each given the list itself: every write reaches the list,
# since the conversion makes a view of it, not a copy.

struct Cursor:
    rows: MutSpan[int]

fn fill(ref mut dst: MutSpan[int], v: int):
    for ref mut x in dst:
        x = v

fn main():
    xs = [1, 2, 3]
    fill(ref mut xs, 9)
    println(xs)
    d: MutSpan[int] = ref mut xs
    d[0] = 7
    println(xs)
    c = Cursor(rows=ref mut xs)
    c.rows[1] = 5
    println(xs)
