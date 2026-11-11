#$ test: run-pass
#$ rules: TYP-25
#$ stdout: 1 2 3
#$ stdout: 1 5 3
#$ stdout: 1 2 9
# `[TYP-25]` — arguments may be positional or named; positional arguments come first; a parameter
# with a default may be omitted.

fn show(a: int, b: int = 2, c: int = 3):
    println(a, b, c)

fn main():
    show(1)
    show(1, c=3, b=5)
    show(1, 2, c=9)
