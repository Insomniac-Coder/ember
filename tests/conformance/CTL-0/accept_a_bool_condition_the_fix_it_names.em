#$ test: run-pass
#$ rules: CTL-0
#$ stdout: 3
#$ stdout: empty
# `[CTL-0]` — the conditions the fix-its give are `bool`s.

fn main():
    n: i32 = 3
    xs: Array[i32] = Array[i32]()
    if n != 0:
        println(n)
    if not xs.is_empty():
        println(1)
    else:
        println("empty")
