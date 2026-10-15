#$ test: run-pass
#$ rules: CLO-10, LEX-16
#$ stdout: 3000000000
#$ 7 2.5
# D-596 — a lambda whose body is a bare literal, with no result type expected, returns the
# literal's default type (`i64`, `f64`): it was `fn() -> an integer`, which converted to no `fn`
# type, and the compiler stopped on the large one.

fn apply[R](owned f: fn() -> R) -> R:
    return f()

fn main():
    x = owned fn() => 3000000000
    println(x())
    y = owned fn() => 7
    g: fn() -> int = y
    println(g(), apply(owned fn() => 2.5))
