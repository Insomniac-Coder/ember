#$ test: run-pass
#$ rules: MNG-3, CG-C-7
#$ profiles: development, release
#$ stdout: 1 2 3
#$ stdout: Pair(café=1, caf_u00E9_=2)
#$ assert-c: contains("caf_u00E9_;")
#$ assert-c: contains("caf_u005F_u00E9_;")
# `[MNG-3]` (ODR-213) — C member names are spelled as symbols are, without the length: the field
# `café` is the C member `caf_u00E9_`, and in the ASCII field `caf_u00E9_` the `_` that would read
# as an escape is one, `caf_u005F_u00E9_`, so one struct holds both. Both were `caf_u00E9_`, one C
# member declared twice. A local keeps its name in the C as a field does (`[CG-C-7]`), so the two
# locals are two C variables too. What the program prints names each as written.

@derive(Debug)
struct Pair:
    café: int
    caf_u00E9_: int

fn total(p: Pair) -> int:
    return p.café + p.caf_u00E9_

fn main():
    p = Pair(1, 2)
    café = p.café
    caf_u00E9_ = p.caf_u00E9_
    println(café, caf_u00E9_, total(p))
    println(p)
