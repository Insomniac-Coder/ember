#$ test: run-pass
#$ rules: LT-1
#$ stdout: a
#$ yy
# ODR-024 — whether a parameter is a source is fixed by the declared signature,
# with each type parameter taken to be `Copy`. `x: T` is not a source even
# when `T` is `String`, so rule 2 ties the result to `xs` alone and the caller
# may assign `s` while `r` lives.

fn pick[T](xs: Array[T], x: T) -> ref T:
    return ref xs[0]

fn main():
    xs: Array[String] = ["a", "b"]
    s: String = "zz"
    r = pick(xs, s)
    s = "yy"
    println(r)
    println(s)
