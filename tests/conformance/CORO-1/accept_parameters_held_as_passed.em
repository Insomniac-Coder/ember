#$ test: run-pass
#$ rules: CORO-1, CORO-6, CORO-7
#$ profiles: debug, development
#$ stdout: hello 2
#$ 11 12
#$ [1, 2, 7]
#$ owned!
#$ 3 3
#$ released
# `[CORO-1]` — the frame holds the parameters as they were passed: a borrowed `String` as the
# caller's (the frame borrows it), an `int` as a copy, a `mut` list as the caller's (the
# generator writes it), an `owned` value as its own, a class handle retained, and released when
# the generator ends.

class Tracker:
    count: int = 0

    fn drop(mut self):
        println("released")

gen fn letters(s: String, n: int) -> Generator[String]:
    yield f"{s} {n}"

gen fn counting(start: int) -> Generator[int]:
    yield start + 1
    yield start + 2

gen fn appending(mut xs: Array[int], x: int) -> Generator[int]:
    xs.push(x)
    yield xs.len()

gen fn consuming(owned s: String) -> Generator[String]:
    s.push_str("!")
    yield s

gen fn tracking(t: Tracker) -> Generator[int]:
    t.count += 3
    yield t.count

fn main():
    text: String = "hello"
    for line in letters(text, 2):
        println(line)
    got = [x for x in counting(10)]
    println(got[0], got[1])
    xs = [1, 2]
    for _ in appending(xs, 7):
        pass
    println(xs)
    for s in consuming("owned"):
        println(s)
    g = tracking(Tracker())
    first = g.next().unwrap()
    println(first, 3)
    for _ in g:
        pass
