#$ test: run-pass
#$ rules: TYP-32
#$ stdout: 4 120
# `[TYP-32]` — every `return` of a `some` function gives the same concrete type, here `int`, which
# `some Display` shows as itself.

fn label(n: int) -> some Display:
    if n > 9:
        return n * 10
    return n

fn main():
    println(label(4), label(12))
