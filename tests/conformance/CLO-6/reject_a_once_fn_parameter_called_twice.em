#$ test: compile-fail
#$ rules: CLO-6
#$ error[E3040]: `f` has been moved out of
# `[CLO-6]` — calling a `once fn` parameter consumes it: a second call is `E3040`.

fn twice(f: once fn() -> int) -> int:
    return f() + f()

fn main():
    println(twice(fn() => 1))
