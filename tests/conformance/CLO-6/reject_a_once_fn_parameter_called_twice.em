#$ test: compile-fail
#$ rules: CLO-6
#$ error[E3040]: `f` is a `once fn`: its call on line 7 uses it up, and it is used again on line 7; a `once fn` runs once (take `f: fn(…)` where it is called more than once)
# `[CLO-6]` — calling a `once fn` parameter consumes it: a second call is `E3040`.

fn twice(f: once fn() -> int) -> int:
    return f() + f()

fn main():
    println(twice(owns fn() => 1))
