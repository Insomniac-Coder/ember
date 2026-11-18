#$ test: run-pass
#$ rules: FN-5
#$ stdout: [9]
#$ stdout: [9]
#$ stdout: x x
#$ stdout: 1
# `[FN-5]` — a default is written for a plain or an `owns` parameter: `fn take(owns xs: Array[int]
# = [])` makes its own list at each call; `t: String = s` reads the earlier `s`, which stays the
# function's own; `copy of s` makes a value of its own for an `owns` parameter.

fn fill(owns xs: Array[int] = []) -> Array[int]:
    xs.push(owns 9)
    return xs

fn twice(s: String, t: String = s) -> String:
    return f"{s} {t}"

fn own(s: String, owns t: String = copy of s) -> int:
    return t.len()

fn main():
    println(fill())
    println(fill())
    println(twice(String("x")))
    println(own(String("y")))
