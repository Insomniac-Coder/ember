#$ test: run-pass
#$ rules: MNG-3, MNG-1
#$ stdout: 5 60
#$ assert-c: contains("int64_t em_u20_u00E9__u005F_u00E9_(")
#$ assert-c: contains("int64_t em_u12_u00E9__u00E9_(")
# `[MNG-3]` (ODR-213, D-862) — inside one component the transliteration never merges two names: the
# ASCII `_` that begins what would read as an escape is itself written `_u005F_`, so the name
# `é_u00E9_` is `_u00E9__u005F_u00E9_` and `éé` is `_u00E9__u00E9_`, two C functions. Both were one
# text, one C function defined twice. Each function is called from two places, so neither is
# inlined into `main`.

fn é_u00E9_(n: int) -> int:
    return n + 1

fn éé(n: int) -> int:
    return n * 10

fn main():
    println(é_u00E9_(1) + é_u00E9_(2), éé(2) + éé(4))
