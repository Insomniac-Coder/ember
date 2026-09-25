#$ test: run-pass
#$ rules: TYP-36, TYP-17
#$ stdout: 0 0.0 false  [] None
# `[TYP-36]` — the `Default` column: zero, `0.0`, `false`, empty text, an
# empty `Array` and `None`, through a `T: Default` bound.

fn make[T: Default]() -> T:
    return T.default()

fn main():
    println(make[int](), make[float](), make[bool](), make[String](), make[Array[int]](), make[Option[int]]())
