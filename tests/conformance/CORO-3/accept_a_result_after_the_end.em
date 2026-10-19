#$ test: run-pass
#$ rules: CORO-3, CORO-1
#$ profiles: debug, development
#$ stdout: none before the end: true
#$ 1 2 3
#$ total 6
#$ taken once: true
# `[CORO-3]` — `Generator[Y, R]`: each `yield` gives a `Y`, and the final `return` an `R` that
# `result()` gives after `next` has returned `None`. Before the end there is none, and the value
# is taken once.

gen fn summing(n: int) -> Generator[int, int]:
    total = 0
    for i in 1..=n:
        total += i
        yield i
    return total

fn main():
    g = summing(3)
    a = g.next().unwrap()
    println("none before the end:", g.result().is_none())
    b = g.next().unwrap()
    c = g.next().unwrap()
    println(a, b, c)
    ended = g.next().is_none()
    if ended:
        println("total", g.result().unwrap())
    println("taken once:", g.result().is_none())
