#$ test: compile-fail
#$ rules: THR-8, THR-9, THR-10, THR-11
# `[THR-8]`, `[THR-9]` (ODR-255) — a `dyn I` whose interface does not require `Send` may hold a
# value that is not (a class handle), so it does not move to another thread (`E7004`); one whose
# interface requires `Send` but not `Sync` is not read by two threads at once (`E7005`); and a type
# that is not `Send` does not implement an interface requiring it (`E2040`).

import std.thread

interface Loose:
    fn value() -> int

interface Sendable: needs Send:
    fn value() -> int

struct Plain:
    n: int

extend Plain implements Loose:
    fn value() -> int:
        return this.n

extend Plain implements Sendable:
    fn value() -> int:
        return this.n

class Handle:
    n: int

    fn init(n: int):
        this.n = n

extend Handle implements Sendable:  #$ error[E2040]: `Sendable` requires `Send`, which `Handle` does not implement
    fn value() -> int:
        return this.n

fn main():
    loose: owns dyn Loose = owns Plain(1)
    h = thread.spawn(owns owned fn() -> int => loose.value())  #$ error[E7004]: captures `loose`, an `owns dyn Loose`, which is not Send
    println(h.join())
    sendable: owns dyn Sendable = owns Plain(2)
    with scope = thread.scope():
        a = scope.spawn(owns fn() => sendable.value())  #$ error[E7005]: borrows `sendable`, an `owns dyn Sendable`, which is not Sync
        println(a.join())
