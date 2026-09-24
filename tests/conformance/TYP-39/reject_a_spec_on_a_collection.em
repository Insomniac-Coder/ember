#$ test: compile-fail
#$ rules: TYP-39, LEX-19
#$ error[E2250]: write `!r` or `!s` first to pad its text
# `[TYP-39]` — as in Python, a list takes no format spec; `!r` and `=` show
# the same text.

fn main():
    xs = [1, 2]
    println(f"{xs:>10}")
