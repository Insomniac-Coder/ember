#$ test: run-pass
#$ rules: GRM-36, BRW-1
#$ stdout: 4 1
#$ stdout: 6
#$ stdout: 9
#$ stdout: 7
## `[GRM-36]` — the operand of `ref` is a place: a local, a field, an index, a call whose result is
## `ref mut T` (`r = ref mut first_even(ref mut xs)`), or, before a call that gives a `MutSpan`, that
## span's elements (`d = ref mut tracks.as_mut_span()`).

struct Pt:
    x: int = 1

fn first_even(ref mut xs: Array[int]) -> ref mut int:
    return ref mut xs[0]

fn main():
    xs = [4, 8]
    p = Pt()
    a = ref xs[0]
    b = ref p.x
    println(a, b)
    r = ref mut first_even(ref mut xs)
    r += 2
    println(xs[0])
    d = ref mut xs.as_mut_span()
    d[1] = 9
    println(xs[1])
    n = 7
    c = ref n
    println(c)
