#$ test: run-pass
#$ rules: MOD-5, WK-11
#$ stdout: 4 0
# D-430 — a program's own `Weak` shadows the compiler's (`[MOD-5]`), and its
# associated functions are its own, `empty` among them. D-401's arm for
# `Weak.empty()` took every `Weak.f()`: `Weak.make()` was `E2060`, "cannot
# infer which `Weak` this is".

struct Weak:
    n: int

extend Weak:
    fn make() -> Weak:
        return Weak(n = 4)

    fn empty() -> Weak:
        return Weak(n = 0)

fn main():
    a = Weak.make()
    b = Weak.empty()
    println(a.n, b.n)
