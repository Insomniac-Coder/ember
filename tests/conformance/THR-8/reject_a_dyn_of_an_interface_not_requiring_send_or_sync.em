#$ test: compile-fail
#$ rules: THR-8, THR-9, THR-10, THR-11
# `[THR-8]`, `[THR-9]` (ODR-255) — a `dyn I` whose interface does not require `Send` may hold a
# value that is not (a class handle), so it does not move to another thread (`E7004`); one whose
# interface requires `Send` but not `Sync` is not read by two threads at once (`E7005`); and a type
# that is not `Send` does not implement an interface requiring it (`E2040`).

import std.thread

interface Loose:
    fn value(self) -> int

interface Sendable: Send:
    fn value(self) -> int

struct Plain:
    n: int

extend Plain implements Loose:
    fn value(self) -> int:
        return self.n

extend Plain implements Sendable:
    fn value(self) -> int:
        return self.n

class Handle:
    n: int

    fn init(mut self, n: int):
        self.n = n

extend Handle implements Sendable:  #$ error[E2040]: `Sendable` requires `Send`, which `Handle` does not implement
    fn value(self) -> int:
        return self.n

fn main():
    loose: Box[dyn Loose] = Box(Plain(1))
    h = thread.spawn(owned fn() -> int => loose.value())  #$ error[E7004]: captures `loose`, a `Box[dyn Loose]`, which is not Send
    println(h.join())
    sendable: Box[dyn Sendable] = Box(Plain(2))
    with scope = thread.scope():
        a = scope.spawn(fn() => sendable.value())  #$ error[E7005]: borrows `sendable`, a `Box[dyn Sendable]`, which is not Sync
        println(a.join())
