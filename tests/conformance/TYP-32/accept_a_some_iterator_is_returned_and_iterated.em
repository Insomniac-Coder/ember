#$ test: run-pass
#$ rules: TYP-32
#$ stdout: 2
#$ stdout: 4
#$ stdout: 6
# `[TYP-32]` — `some I` in a return type is one concrete type the body chooses (here a filter over a
# span's items, whose lambda's type cannot be written), which the caller sees only as `I`. The
# specification's own example, iterated, and handed to a generic function bounded by `I`.

fn evens(xs: Span[int]) -> some Iterator[Item = int]:
    return xs.iter().copied().filter(fn(x) => x % 2 == 0)

fn total[I: Iterator[Item = int]](owned it: I) -> int:
    s = 0
    for x in it:
        s += x
    return s

fn main():
    a = [1, 2, 3, 4]
    for x in evens(a.as_span()):
        println(x)
    println(total(evens(a.as_span())))
