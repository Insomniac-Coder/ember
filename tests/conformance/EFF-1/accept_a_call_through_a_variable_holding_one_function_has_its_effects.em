#$ test: run-pass
#$ rules: EFF-1, FN-6, CLO-1
#$ profiles: debug, development, release
#$ stdout: 2 2 15
# ODR-278 (the owner's example, D-901), `[EFF-1]` — `f = add_one` gives `f` the type of
# `add_one` itself, not "some `fn(int) -> int`", so a call through `f` is a call of `add_one`
# and has its effects only: `fast` is `@noalloc` though `make_list`, another `fn(int) -> int`
# used as a value, allocates. It was `E4001` ("fast -> make_list -> a new Array"). A lambda
# held in a variable, and a function given to a generic, are called as exactly.

fn add_one(x: int) -> int:
    return x + 1

fn make_list(x: int) -> int:
    return [x, x].len()

fn twice[F: fn(int) -> int](f: F, x: int) -> int:
    return f(f(x))

@noalloc
fn fast(x: int) -> int:
    f = add_one
    return f(x)

fn slow(x: int) -> int:
    g = make_list
    return g(x)

@noalloc
fn lambda_held(x: int) -> int:
    k = fn(y: int) -> int => y * 3 + 2
    return twice(k, x) - twice(add_one, x) + 1

fn main():
    println(fast(1), slow(2), lambda_held(1))
