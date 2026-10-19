#$ test: run-pass
#$ rules: CORO-1, CORO-3
#$ profiles: debug, development
#$ stdout: made
#$ started
#$ 1
#$ resumed
#$ 2
#$ ending
#$ done true true
# `[CORO-1]` — calling a generator runs none of its body: the frame is made, and each `next`
# runs the body to its next `yield`. `[CORO-3]` — at the end `next` gives `None`, and again
# after that.

gen fn noisy(first: int) -> Generator[int]:
    println("started")
    yield first
    println("resumed")
    yield first + 1
    println("ending")

fn main():
    g = noisy(1)
    println("made")
    println(g.next().unwrap())
    println(g.next().unwrap())
    ended = g.next().is_none()
    println("done", ended, g.next().is_none())
