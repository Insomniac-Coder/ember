#$ test: compile-fail
#$ rules: THR-10, THR-8
#$ error[E7004]: the thread's function captures `count`, a `Shared[i64]`, which is not Send
#$ error[E3063]: the thread's function captures `view`, a view (`Span[i64]`)
#$ error[E3063]: the thread's function captures `total`, a view (`ref i64`)
#$ error[E7004]: the thread's function returns a `Plain`, which is not Send
#$ error[E7004]: a `fn() -> i64` value may hold anything, so it is not Send
#$ error[E2020]: `thread.spawn` takes a function of no arguments, found `fn(i64) -> i64`
# `[THR-10]` — what a thread's function captures and returns must be `Send` (`E7004`, naming the
# capture); it may capture no view of a local, which the thread could outlive (`E3063`), and a
# lambda that borrows is one; a callable value's captures are unknown, so it is not `Send`.

import std.thread

class Plain:
    n: int

fn main():
    count = Shared(0)
    a = thread.spawn(owned fn() => count.get() + 1)
    xs = [1, 2, 3]
    view: Span[int] = xs
    b = thread.spawn(owned fn() => view.len())
    total = 5
    c = thread.spawn(fn() => total + 1)
    d = thread.spawn(owned fn() => Plain(3))
    g: fn() -> int = owned fn() => 4
    e = thread.spawn(g)
    f = thread.spawn(owned fn(x: int) => x)
