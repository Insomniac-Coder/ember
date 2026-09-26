#$ test: run-pass
#$ rules: TYP-36, LEX-19, STD-9
#$ stdout: ()
#$ stdout: ()|()|  ()
#$ stdout: false true true ()
# `[TYP-36]` — `void` is `Eq` and `Ord` and has no `Display`; printing shows its `Debug`,
# `()`, and `!r` or `!s` make it that text before a spec pads it (D-355).

fn main():
    println(())
    x = ()
    println(f"{x}|{x!r}|{x!r:>4}")
    println(() < (), () <= (), x == (), min((), ()))
