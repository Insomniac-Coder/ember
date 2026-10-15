#$ test: run-pass
#$ rules: CLO-10
#$ stdout: 6
# `[CLO-10]` — up to three words of captured state are held inline, with no allocation.

@noalloc
fn three(a: int, b: int, c: int) -> fn() -> int:
    return owned fn() => a + b + c

fn main():
    f = three(1, 2, 3)
    println(f())
