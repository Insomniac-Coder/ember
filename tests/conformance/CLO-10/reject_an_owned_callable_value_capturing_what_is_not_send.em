#$ test: compile-fail
#$ rules: CLO-10, THR-8, THR-10
# `[CLO-10]`, `[THR-8]` (ODR-218) — an owned callable value is `Send` only when everything it
# captures is: one capturing a handle of a class that is not `@sync` stays on its thread
# (`E7004`, naming the capture), and so does a type parameter whose bounds do not require `Send`.

import std.thread

class Counter:
    n: int

    fn init(mut self):
        self.n = 0

fn run_later[F: fn() -> int](owned task: F) -> int:
    h = thread.spawn(owned fn() -> int => task())  #$ error[E7004]: captures `task`, a `F`, which is not Send
    return h.join()

fn main():
    c = Counter()
    peek = owned fn() -> int => c.n
    h = thread.spawn(peek)  #$ error[E7004]: captures `c`, a `Counter`, which is not Send
    println(h.join(), run_later(owned fn() -> int => 1))
