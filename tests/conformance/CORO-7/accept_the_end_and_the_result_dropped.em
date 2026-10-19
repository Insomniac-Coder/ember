#$ test: run-pass
#$ rules: CORO-7, CORO-3
#$ profiles: debug, development
#$ stdout: 1
#$ drop kept
#$ 2
#$ drop result
#$ drop local
#$ end
# `[CORO-7]` — a finished generator still holds its result until `result()` takes it or the frame
# is dropped; a suspended one drops only what is live where it is suspended (ADR-193: a pending
# `defer` does not run, as no drop is one).

import std.mem

struct Noisy:
    name: str

    fn drop(mut self):
        println("drop", self.name)

gen fn finishing() -> Generator[int, Noisy]:
    yield 1
    return Noisy("kept")

gen fn returning() -> Generator[int, Noisy]:
    yield 2
    return Noisy("result")

gen fn deferring() -> Generator[int]:
    local = Noisy("local")
    defer:
        println("deferred", local.name)
    yield 3

fn print_all[G: Iterator[Item = int]](mut g: G):
    while true:
        match g.next():
            Some(x):
                println(x)
            None:
                break

fn main():
    g = finishing()
    print_all(g)
    taken = g.result()
    mem.drop(taken)
    h = returning()
    print_all(h)
    mem.drop(h)
    d = deferring()
    d.next()
    mem.drop(d)
    println("end")
