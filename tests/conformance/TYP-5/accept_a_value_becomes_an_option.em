#$ test: run-pass
#$ rules: TYP-5
#$ profiles: debug, release, shipping
#$ stdout: 1
#$ true false
#$ 7 3 hi
# `[TYP-5]` rule 11 — a `T` becomes `Option[T]` as `Some(value)` at a coercion
# site, after widening and the text rules.

fn find(xs: Array[int], want: int) -> Option[int]:
    for i, x in enumerate(xs):
        if x == want:
            return i
    return None

fn present(x: Option[int]) -> bool:
    return x.is_some()

fn main():
    println(find([4, 5], 5).unwrap())
    println(present(5), present(None))
    y: Option[int] = 7
    n: i32 = 3
    z: Option[i64] = n
    s: Option[String] = "hi"
    println(y.unwrap(), z.unwrap(), s.unwrap())
