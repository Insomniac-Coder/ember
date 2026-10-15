#$ test: run-pass
#$ rules: EFF-19
#$ stdout: 9
# `[EFF-19]` — a function with none of `@realtime`'s effects keeps it; printing is not among them.

@realtime
fn scale(x: int) -> int:
    return x * 3

fn main():
    println(scale(3))
