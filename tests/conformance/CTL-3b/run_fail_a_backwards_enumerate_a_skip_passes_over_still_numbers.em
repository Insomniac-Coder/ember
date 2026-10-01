#$ test: run-fail
#$ rules: CTL-3b, TYP-8
#$ profiles: debug, release
#$ panics: integer overflow in `+`
# `[CTL-3b]` — a backwards `enumerate` numbers its greatest item first
# (`Enumerate.next_back`), and `skip` pulls the items it passes over: here
# the loop gets no turn, but numbering the first pulled item, `int.MAX - 1 +
# 2`, overflows, as it does through std's adapters (D-457).

fn main():
    xs: Array[int] = [1, 2, 3]
    top = int.MAX - 1
    for i, x in xs.iter().copied().enumerate(start=top).rev().skip(3):
        println(i, x)
    println("no turn")
