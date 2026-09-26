#$ test: compile-fail
#$ rules: TYP-15, FN-1
# `[TYP-15]` (D-352) — a `mut` parameter is the caller's place, which outlives
# the call, so a view of the callee's own local may not be stored in it.

fn fill(mut out: str):
    buf = String.from("temporary")
    out = buf.as_str() #$ error[E3063]: a view of `buf` is stored in `out`, which the caller owns, so it would outlive `buf`

fn main():
    name = "ann"
    fill(name)
    println(name)
