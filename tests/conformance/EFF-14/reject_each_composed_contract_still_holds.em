#$ test: compile-fail
#$ rules: EFF-14, EFF-5, EFF-16
#$ error[E4001]: `@noalloc` function `grow` reaches an allocation
#$ error[E4040]: `@nopanic(explicit)` function `stop` reaches a panic
# `[EFF-14]` — contracts compose, and each one in the set is kept on its own: a function with the
# whole inner-loop set is refused for an allocation and for a panic it writes.

@no_alias_checks
@noalloc
@nosync
@nopanic(explicit)
fn grow(ref mut xs: Array[int]):
    xs.push(owns 1)

@no_alias_checks
@noalloc
@nosync
@nopanic(explicit)
fn stop(n: int) -> int:
    if n < 0:
        panic("negative")
    return n

fn main():
    xs: Array[int] = []
    grow(ref mut xs)
    println(stop(1))
