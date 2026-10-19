#$ test: run-pass
#$ rules: CORO-10, FN-6
#$ profiles: debug, development
#$ stdout: 6
# `[CORO-10]` — within Ember a `gen fn` is a function like any other: a value that, called,
# builds the frame.

gen fn upto(n: int) -> Generator[int]:
    for i in 1..=n:
        yield i

fn total_of[G: Iterator[Item = int]](make: fn(int) -> G) -> int:
    t = 0
    for x in make(3):
        t += x
    return t

fn main():
    println(total_of(upto))
