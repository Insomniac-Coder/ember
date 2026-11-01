#$ test: run-pass
#$ rules: OWN-9, TYP-38, CORO-1
#$ stdout: seven 7
#$ 3
#$ 3
# `[OWN-9]` — only an `owns` argument, `return` and `b = owns n` hand a number over: a collection
# literal and a `yield` give theirs as `=` does, a copy.

gen fn twice(n: int) -> Generator[int]:
    i = n
    yield i
    yield i

fn main():
    k = 7
    names: Map[int, StrView] = {k: "seven"}
    println(names[k], k)
    for v in twice(3):
        println(v)
