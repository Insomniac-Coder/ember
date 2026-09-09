#$ test: run-pass
#$ rules: GRM-17, PHIL-8a
## `[PHIL-8a]` — "the mandated `help`, applied literally to the rejected
## program, produces a program that compiles". This is that program.

fn apply(f: fn(i32) -> i32, v: i32) -> i32:
    return f(v)

fn main():
    h: fn(i32) -> i32 = fn(x) => x * 2
    println(apply(h, 4))
#$ stdout: 8
