#$ test: run-pass
#$ rules: MNG-3
#$ assert-c: contains(caf_u00E9_)
#$ assert-c: contains(Ma_u00DF_)
#$ stdout: 5
# `[MNG-3]` — a character beyond ASCII in a name is `_uXXXX_` in C, its code point in hex: `café`
# is `caf_u00E9_`, `Maß` is `Ma_u00DF_`, so every C compiler takes the identifier.

struct Maß:
    n: int

fn café(m: Maß) -> int:
    return m.n + 1

fn main():
    a = Maß(n = 1)
    b = Maß(n = 2)
    println(café(a) + café(b))
