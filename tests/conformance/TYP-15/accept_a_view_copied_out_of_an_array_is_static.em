#$ test: run-pass
#$ rules: TYP-15, LT-3
#$ stdout: ann ['ann', 'bob', 'cy']
# `[TYP-15]` (D-198, ODR-069) — an `Array`'s elements hold only `static`
# views, so a view copied out of one is `static`, even read through a
# reference to the array: `x` does not keep `names` borrowed, and `names`
# may grow while `x` is used.

fn main():
    names = ["ann", "bob"]
    r = ref names
    x = r[0]
    names.push("cy")
    println(x, names)
