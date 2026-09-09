#$ test: run-pass
#$ rules: CLO-1, TYP-23
## "It is a plain value type if declared `owned fn` (captures by
## move/copy/retain) **or captures nothing**." `[TYP-23]` rule 4 — the
## parameter types come from the expected function type.

fn apply(f: fn(i32) -> i32, v: i32) -> i32:
    return f(v)

fn main():
    println(apply(fn(x) => x * 2, 21))
    println(apply(fn(x: i32) => x + 1, 41))
    step: fn(i32) -> i32 = fn(x) => x * 3
    println(apply(step, 4))
#$ stdout: 42
#$ 42
#$ 12
