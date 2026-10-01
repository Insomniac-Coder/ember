#$ test: compile-fail
#$ rules: WK-11, TYP-5
# D-434 — at a place whose type is known and is no `Weak`, `Weak.empty()` is
# a mismatch naming that type. It said "cannot infer which `Weak` this is"
# and to give the place a type, which it had.

fn main():
    b: int = Weak.empty()    #$ error[E2020]: expected `i64`, found a `Weak`
    println(b)
