#$ test: run-pass
#$ rules: GRM-25
#$ profiles: debug, release, shipping
#$ stdout: in
#$ true
#$ false
#$ called
#$ true
#$ false
# `a < b <= c` is `a < b and b <= c`, left to right, stopping at the first
# false comparison: the last operand is not evaluated when an earlier link
# fails, and is evaluated once when every earlier link holds.

fn limit() -> int:
    println("called")
    return 20

fn main():
    q = 5
    if 0 <= q < 10:
        println("in")
    println(1 < 2 < 3 < 4)
    println(1 < 3 < 2)
    println(4 < q <= limit())
    println(9 < q < limit())
