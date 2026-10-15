#$ test: run-pass
#$ rules: STD-2
#$ stdout: hi
#$ 3
# `[STD-2]` — `println("literal")`, `println(some_str)` and `println(n)` for a number are
# `@noalloc`.

@noalloc
fn report(s: str, n: int):
    println(s)
    println(n)

fn main():
    report("hi", 3)
