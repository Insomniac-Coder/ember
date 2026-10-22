#$ test: run-pass
#$ rules: CLO-2, CLO-1
#$ profiles: debug, development, release
#$ stdout: 7 6
#$ 12
## `[CLO-2]` — a lambda captures the enclosing function's locals, and so does a lambda inside a
## lambda: the outer one captures them for it, shared or mutable as the inner one uses them (the
## inner one's names were `E1010`). A capture the program calls `env` is the program's (it was the
## closure's own environment).

fn main():
    k = 3
    total = 0
    name = "ab".to_string()
    f = fn(x: int) -> int:
        g = fn(y: int) => x * y + k + name.len()
        return g(2)
    bump = fn():
        inner = fn(): total += k
        inner()
        inner()
    bump()
    println(f(1), total)
    env = 5
    other = 7
    h = fn() => env + other
    println(h())
