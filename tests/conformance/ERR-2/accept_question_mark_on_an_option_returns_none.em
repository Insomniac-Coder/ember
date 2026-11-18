#$ test: run-pass
#$ rules: ERR-2
#$ stdout: Some(7)
#$ stdout: None
#$ stdout: Some(10)
#$ stdout: None
# `[ERR-2]` — `e?` on an `Option[T]` in a function returning `Option[U]` evaluates to the `Some`
# payload, or returns `None`.

fn first(xs: Array[int]) -> Option[int]:
    if xs.len() == 0:
        return None
    return Some(xs[0])

fn plus_one(xs: Array[int]) -> Option[int]:
    v = first(xs)?
    return Some(v + 1)

fn sum_two(xs: Array[int]) -> Option[int]:
    a = xs.get(0)?
    b = xs.get(1)?
    return Some(a + b)

fn main():
    println(plus_one([6]))
    println(plus_one([]))
    println(sum_two([4, 6]))
    println(sum_two([4]))
