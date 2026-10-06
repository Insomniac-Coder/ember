#$ test: run-pass
#$ rules: CTL-3b, STD-19
#$ profiles: debug, release
#$ stdout: no turn
# `[CTL-3b]` — `zip` asks its first side for an item before the second, so a backwards
# `enumerate` zipped with an empty list is still pulled, and numbers its greatest item, though the
# loop gets no turn (D-457). Its numbers are `u64`s, which hold it (`[STD-19]`, the owner's ruling
# of 2026-10-06): nothing overflows.

fn main():
    xs: Array[int] = [1, 2, 3]
    ys: Array[int] = []
    top = int.MAX - 1
    for p, q in xs.iter().copied().enumerate(start=top).rev().zip(ys.iter().copied()):
        println(p, q)
    println("no turn")
