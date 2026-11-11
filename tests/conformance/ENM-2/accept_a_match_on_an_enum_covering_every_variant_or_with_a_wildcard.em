#$ test: run-pass
#$ rules: ENM-2
#$ stdout: 2 0
# `[ENM-2]` — a `match` on an enum MUST be exhaustive; `_` covers the rest.

enum Color:
    Red
    Green
    Blue

fn code(c: Color) -> int:
    return match c:
        Color.Red => 1
        Color.Green => 2
        Color.Blue => 3

fn is_red(c: Color) -> int:
    return match c:
        Color.Red => 1
        _ => 0

fn main():
    println(code(Color.Green), is_red(Color.Blue))
