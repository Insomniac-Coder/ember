#$ test: run-pass
#$ rules: CLO-3, CLO-6, OWN-3, TST-19
#$ profiles: debug, development, release
#$ stdout: 14 21 6
# `[CLO-6]` (D-850) — only `once fn` is single-call: an `owned f: fn(A) -> R` parameter is taken
# over and called any number of times, in a loop too (it was `E3040` for a second call and
# `E3041` in a loop, as if it were `once fn`).

fn twice(owned f: fn() -> i32) -> i32:
    first = f()
    return first + f()

fn thrice(owned f: fn(i32) -> i32, x: i32) -> i32:
    total: i32 = 0
    for _ in 0..3:
        total += f(x)
    return total

fn count_calls(owned f: fn()) -> int:
    calls = 0
    while calls < 6:
        f()
        calls += 1
    return calls

fn main():
    n: i32 = 7
    println(twice(fn() => n), thrice(fn(x) => x + n, 0), count_calls(fn(): pass))
