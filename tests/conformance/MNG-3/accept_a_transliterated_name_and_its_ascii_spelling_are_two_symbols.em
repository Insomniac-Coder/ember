#$ test: run-pass
#$ rules: MNG-3, MNG-1
#$ stdout: 5 60
#$ assert-c: contains("int64_t em_u10caf_u00E9_(")
#$ assert-c: contains("int64_t em_u16caf_u005F_u00E9_(")
#$ assert-c: contains("struct em_u9Ma_u00DF_ {")
#$ assert-c: contains("struct em_u15Ma_u005F_u00DF_ {")
#$ assert-c: contains("{"em_u10caf_u00E9_", "caf\xc3")
#$ assert-c: contains("{"em_u16caf_u005F_u00E9_", "caf_u00E9_", ")
# `[MNG-3]` (ODR-202, ODR-213) — a path component that was transliterated has `u` before its length:
# `café` is `u10caf_u00E9_`; in the ASCII name spelled `caf_u00E9_` the `_` that would read as an
# escape is one, `_u005F_`, so it is `u16caf_u005F_u00E9_`, and the two are two symbols (`[MNG-1]`:
# two distinct items never share a name). Without the mark both functions were `em_10caf_u00E9_`,
# one C function defined twice, and both types one C struct. Each function is
# called from two places, so neither is inlined into `main`, and each frame is named by its own
# path (`[CG-C-10]`; `café` in UTF-8, `caf\xc3\xa9`).

struct Maß:
    n: int

struct Ma_u00DF_:
    n: int

fn café(m: Maß) -> int:
    return m.n + 1

fn caf_u00E9_(m: Ma_u00DF_) -> int:
    return m.n * 10

fn main():
    println(café(Maß(n = 1)) + café(Maß(n = 2)), caf_u00E9_(Ma_u00DF_(n = 2)) + caf_u00E9_(Ma_u00DF_(n = 4)))
