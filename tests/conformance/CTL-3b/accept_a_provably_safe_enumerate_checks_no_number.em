#$ test: run-pass
#$ rules: CTL-3b, STD-19, RNG-4, HEAP-8
#$ profiles: debug, release, shipping
#$ stdout: 4 8 12 16
#$ assert-c: !contains("integer overflow in")
# `[CTL-3b]` — the numbers `enumerate(start=round)` gives fit in `int` when
# the list has no more items than the room above `round`. A list of 8-byte
# numbers has at most `PTRDIFF_MAX / 8` items (`[HEAP-8]`) and `round` is
# below 3, so the range facts decide that test: no number is checked, and
# the loop has no check at its end (only `x += i` keeps its own).

fn main():
    xs: Array[int] = [1, 2, 3, 4]
    for round in 0..3:
        for (i, x) in xs.iter_mut().enumerate(start=round):
            x += i
    println(f"{xs[0]} {xs[1]} {xs[2]} {xs[3]}")
