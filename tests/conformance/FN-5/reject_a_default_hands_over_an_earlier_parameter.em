#$ test: compile-fail
#$ rules: FN-5
# `[FN-5]` — a default may read an earlier parameter and neither changes it nor hands it over
# (`E3013`, as a plain name): `t: String = s` reads `s`, which stays the function's own, and an
# `owns` parameter whose default is `s` would hand `s` over.

fn keep(s: String, owns t: String = s):    #$ error[E3013]
    println(t)

fn main():
    keep(String("x"))
