#$ test: run-pass
#$ rules: EXP-4, CTL-3b
#$ stdout:
#$ 1
#$ 2
#$ 0 1
#$ 1 2
#$ 1
#$ 2
#$ 0 1
#$ 1 2
#$ 4
#$ 5
# `[EXP-4]` — a temporary a `for` iterable borrows lives to the end of the
# loop, when the view is a call's result too: `head(make())` views the list
# `make()` returns, which stays alive for the whole loop, with or without
# adapters (it was `E3060`/`E3020`).

fn head(xs: Array[int]) -> Span[int]:
    return xs[..2]

fn make() -> Array[int]:
    return [1, 2, 3]

fn main():
    for x in head(make()).iter().take(2):
        println(x)
    for i, x in head(make()).iter().enumerate():
        println(i, x)
    for x in head(make()):
        println(x)
    for i, x in enumerate(head(make())):
        println(i, x)
    for x in [4, 5]:
        println(x)
