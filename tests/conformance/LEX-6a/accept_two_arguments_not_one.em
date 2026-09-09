#$ test: run-pass
#$ rules: LEX-6a
## "Consequently `f(fn(): g(), h())` passes two arguments; `h()` is not part
## of the lambda."

fn pick(f: fn(i32) -> i32, v: i32) -> i32:
    return f(v)

fn one() -> i32:
    return 1

fn main():
    println(pick(fn(x: i32): return x + one(), 41))
#$ stdout: 42
