#$ test: compile-fail
#$ rules: CLO-10, THR-8, THR-9, FN-6
# ODR-264 — a `fn(A) -> R + Send` value holds only what is `Send`, checked where it is made: a
# lambda capturing a one-thread owner is refused (`E7015`, naming the capture);
# so is a `fn(A) -> R` value, which may hold anything, and a callable parameter whose type does not
# say `+ Send`; a parameter written `f: fn() -> int + Send` takes only what is `Send`. A call may
# change what a callable value captured, so one is never `Sync` (`E7005`).

import std.thread

class Counter:
    n: int

    fn init():
        this.n = 0

fn later(owns f: fn() -> int + Send) -> int:
    h = thread.spawn(owns owned fn() -> int => f())
    return h.join()

fn widen(owns f: fn() -> int) -> fn() -> int + Send:
    return f  #$ error[E7004]: this callable parameter may hold anything

fn main():
    c = shares Counter()
    peek: fn() -> int + Send = owned fn() -> int => c.n  #$ error[E7015]: captures `c`, a `shares Counter`, which is not Send
    held: fn() -> int = owned fn() -> int => 1
    sent: fn() -> int + Send = held  #$ error[E7004]: may hold anything, so it is not a `fn() -> i64 + Send`
    println(later(owns owned fn() -> int => c.n))  #$ error[E7015]: this lambda captures `c`, a `shares Counter`, which is not Send
    shared: fn() + Sync = owned fn() => println(1)  #$ error[E7005]: a callable type is never `Sync`
