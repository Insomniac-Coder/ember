#$ test: run-pass
#$ rules: CORO-6, CORO-5, CORO-4
#$ profiles: debug, release
#$ stdout: 1
#$ 2 3
#$ 4
#$ ab
# `[CORO-6]` — because no borrow of the frame's own locals outlives a `yield`, a suspended
# generator may move: returned from the function that started it, put in a list, taken out
# again, and resumed where it stopped, its `String` with it (`[CORO-4]`).

gen fn counting(n: int) -> Generator[int]:
    i = 1
    while i <= n:
        yield i
        i += 1

gen fn growing() -> Generator[String]:
    s: String = "a"
    yield s.clone()
    s.push_str("b")
    yield s.clone()

fn started[G: Iterator[Item = int]](owned g: G) -> G:
    println(g.next().unwrap())
    return g

fn main():
    g = started(counting(4))
    a = g.next().unwrap()
    b = g.next().unwrap()
    println(a, b)
    parked = [g]
    h = parked.pop().unwrap()
    for x in h:
        println(x)
    w = growing()
    w.next()
    moved = w
    println(moved.next().unwrap())
