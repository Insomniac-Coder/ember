#$ test: run-pass
#$ rules: SPN-2, EXP-1
#$ profiles: debug, development, release
#$ stdout: 8 [1, 2, 3, 4, 5, 6, 7, 8]
#$ [4, 5, 6, 7, 8]
#$ he
# `[EXP-1]` — a slice's view is evaluated once, before its bounds. A bound
# that changes the viewed place changes neither what is sliced nor the length
# the bounds are checked against (a review found the length read before the
# bound and the pointer after it: an out-of-bounds read).

fn shrink(ref mut v: Span[int]) -> int:
    v = v[7..]
    return 8

fn cut(ref mut v: Span[int]) -> int:
    v = v[5..]
    return 3

fn swap_text(ref mut s: StrView) -> int:
    s = "abcdefgh"
    return 2

fn main():
    xs = [1, 2, 3, 4, 5, 6, 7, 8]
    v: Span[int] = xs
    w = v[0..shrink(ref mut v)]
    println(w.len(), w)
    u: Span[int] = xs
    println(u[cut(ref mut u)..])
    s: StrView = "hello"
    println(s[0..swap_text(ref mut s)])
