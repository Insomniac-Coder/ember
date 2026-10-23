#$ test: compile-fail
#$ rules: CLO-10, THR-8, THR-9, FN-6
# ODR-264 — a `fn(A) -> R + Send` value holds only what is `Send`, checked where it is made: a
# lambda capturing a handle of a class that is not `@sync` is refused (`E7004`, naming the capture);
# so is a `fn(A) -> R` value, which may hold anything, and a callable parameter whose type does not
# say `+ Send`; a parameter written `f: fn() -> int + Send` takes only what is `Send`. A call may
# change what a callable value captured, so one is never `Sync` (`E7005`).

import std.thread

class Counter:
    n: int

    fn init(mut self):
        self.n = 0

fn later(owned f: fn() -> int + Send) -> int:
    h = thread.spawn(owned fn() -> int => f())
    return h.join()

fn widen(owned f: fn() -> int) -> fn() -> int + Send:
    return f  #$ error[E7004]: this callable parameter may hold anything

fn main():
    c = Counter()
    peek: fn() -> int + Send = owned fn() -> int => c.n  #$ error[E7004]: captures `c`, a `Counter`, which is not Send
    held: fn() -> int = owned fn() -> int => 1
    sent: fn() -> int + Send = held  #$ error[E7004]: may hold anything, so it is not a `fn() -> i64 + Send`
    println(later(owned fn() -> int => c.n))  #$ error[E7004]: this lambda captures `c`, a `Counter`, which is not Send
    shared: fn() + Sync = owned fn() => println(1)  #$ error[E7005]: a callable type is never `Sync`
