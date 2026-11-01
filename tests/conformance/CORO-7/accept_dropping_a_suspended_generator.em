#$ test: run-pass
#$ rules: CORO-7, CORO-4
#$ profiles: debug, development
#$ stdout: got 1
#$ drop second
#$ drop first
#$ unstarted
#$ drop param
#$ finished 2 third
#$ drop third
#$ end
# `[CORO-7]` — dropping a suspended generator drops exactly the locals live at its suspension
# point, in reverse declaration order: here two, then (never started) the owned parameter, then
# none, the generator having run to its end and dropped its own.

import std.mem

struct Noisy:
    name: StrView

extend Noisy:
    fn drop():
        println("drop", this.name)

gen fn two() -> Generator[int]:
    first = Noisy("first")
    second = Noisy("second")
    yield 1
    yield 2
    println(first.name, second.name)

gen fn holds(owns n: Noisy) -> Generator[int]:
    yield 1
    println(n.name)

gen fn runs() -> Generator[int]:
    third = Noisy("third")
    yield 2
    println("finished", 2, third.name)

fn main():
    g = two()
    println("got", g.next().unwrap())
    mem.drop(g)
    h = holds(owns Noisy("param"))
    println("unstarted")
    mem.drop(h)
    for _ in runs():
        pass
    println("end")
