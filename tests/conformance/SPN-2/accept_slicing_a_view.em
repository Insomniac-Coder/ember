#$ test: run-pass
#$ rules: SPN-2
#$ profiles: debug, release, shipping
#$ stdout: [20, 30] [10, 20] [30, 40] [10, 20, 30, 40] [20, 30]
#$ 90
#$ [20]
#$ [2, 3]
#$ 30
#$ 40
# Part VI's slice row — `a[i..j]`, `a[..j]`, `a[i..]` view part of an `Array`,
# a fixed array or a `Span` as a `Span`, bounds-checked.

fn total(xs: Span[int]) -> int:
    return sum(xs)

fn main():
    xs = [10, 20, 30, 40]
    println(xs[1..3], xs[..2], xs[2..], xs[..], xs[1..=2])
    println(total(xs[1..]))
    view: Span[int] = xs
    println(view[1..2])
    fixed: [int; 3] = [1, 2, 3]
    println(fixed[1..])
    for x in xs[2..]:
        println(x)
