#$ test: run-pass
#$ rules: STD-19, TYP-17
#$ stdout: 6
#$ stdout: 2

## D-735 — `it.copied()` and `it.cloned()` in a generic body asked the type table whether `T` is
## `Copy` or `Clone`, which knows nothing of `T`'s bounds: `T: Copy` was "`copied` needs `T` to be
## `Copy`".

[T: needs (Copy, Add[Output = T])]
fn total(xs: Span[T], zero: T) -> T:
    s = zero
    for x in xs.iter().copy_each():
        s = s + x
    return s

[T: needs Clone]
fn count(xs: Span[T]) -> int:
    n = 0
    for _ in xs.iter().copy_each():
        n += 1
    return n

fn main():
    a = [1, 2, 3]
    println(total(a.as_span(), 0))
    b = ["x", "y"]
    println(count(b.as_span()))
