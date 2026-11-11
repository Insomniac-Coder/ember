#$ test: run-pass
#$ rules: ENM-4, STR-5
#$ stdout: 7 7
# `[ENM-4]` — a payload enum is `Copy` only by `@derive(Copy)` with every payload `Copy`.

@derive(Copy)
enum Num:
    Int(i32)
    Nothing

fn show(n: Num) -> int:
    return match n:
        Num.Int(v) => v
        Num.Nothing => 0

fn main():
    a = Num.Int(7)
    b = a
    println(show(a), show(b))
