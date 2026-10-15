#$ test: run-pass
#$ rules: CLO-6, FN-6, CLO-3
#$ stdout: 5 5 8
# `[CLO-6]` — a parameter `f: once fn(A) -> R` accepts a closure that gives away a capture (callable
# only once) and one that does not; it is a bound like any callable parameter (`[CLO-3]`), so a
# lambda that borrows the caller's locals is accepted too, and calling `f` consumes it.

fn consume(owned s: String) -> int:
    return s.len()

fn call_once(f: once fn() -> int) -> int:
    return f()

fn apply(f: once fn(int) -> int, x: int) -> int:
    return f(x)

interface Runner:
    fn run(self, f: once fn() -> int) -> int

struct Plus:
    n: int

extend Plus implements Runner:
    fn run(self, f: once fn() -> int) -> int:
        return f() + self.n

fn main():
    s = "hello".to_string()
    n = 3
    println(call_once(owned fn() => consume(s)), apply(fn(x) => x + n, 2), Plus(1).run(fn() => n + 4))
