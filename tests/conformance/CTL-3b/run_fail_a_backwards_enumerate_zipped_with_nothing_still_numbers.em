#$ test: run-fail
#$ rules: CTL-3b, TYP-8
#$ profiles: debug, release
#$ panics: integer overflow in `+`
# `[CTL-3b]` — `zip` asks its first side for an item before the second, so a
# backwards `enumerate` zipped with an empty list is still pulled, and
# numbering its greatest item overflows, though the loop gets no turn: as
# through std's adapters (D-457).

fn main():
    xs: Array[int] = [1, 2, 3]
    ys: Array[int] = []
    top = int.MAX - 1
    for p, q in xs.iter().copied().enumerate(start=top).rev().zip(ys.iter().copied()):
        println(p, q)
    println("no turn")
