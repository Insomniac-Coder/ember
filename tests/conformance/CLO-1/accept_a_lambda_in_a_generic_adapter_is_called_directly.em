#$ test: run-pass
#$ rules: CLO-1, FN-6, STD-19, CLO-14
#$ profiles: development, release
#$ stdout: 12 7 9
#$ assert-c: !contains("ember_fn_value")
#$ assert-c: !contains("native_fn_value")
# ODR-278, `[CLO-1]` — a lambda that captures nothing, and a function, each have a type of their
# own, which a generic argument keeps: `map(f)` is a `Map` made for `f`, whose `next` calls `f`
# directly. No callable value is made, held or called through (it was a 40-byte
# `ember_fn_value` in the adapter, called through its entry), whether the adapter is used where
# it is made or kept in a variable, and a function held in a local is called as itself.

fn add_one(x: int) -> int:
    return x + 1

fn main():
    xs = [1, 2, 3]
    total = xs.iter().map(fn(x: ref int) -> int => x * 2).sum()
    kept = xs.iter().map(add_one).filter(fn(x: int) -> bool => x > 2)
    rest = 0
    for v in kept:
        rest += v
    f = add_one
    println(total, rest, f(8))
