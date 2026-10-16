#$ test: compile-fail
#$ rules: THR-10, THR-8
#$ error[E7004]: the thread's function captures `count`, a `Shared[i64]`, which is not Send
#$ error[E3026]: the thread's function borrows `total`, which it may outlive
#$ error[E7004]: the thread's function returns a `Plain`, which is not Send
#$ error[E7004]: a `fn() -> i64` value may hold anything, so it is not Send
#$ error[E2020]: `thread.spawn` takes a function of no arguments, found `fn(i64) -> i64`
# `[THR-10]` — what a thread's function captures and returns must be `Send` (`E7004`, naming the
# capture); a lambda that borrows may be outlived by the thread (`E3026`, ODR-103); a callable
# value's captures are unknown, so it is not `Send`.

import std.thread

class Plain:
    n: int

fn main():
    count = Shared(0)
    a = thread.spawn(owned fn() => count.get() + 1)
    total = 5
    c = thread.spawn(fn() => total + 1)
    d = thread.spawn(owned fn() => Plain(3))
    g: fn() -> int = owned fn() => 4
    e = thread.spawn(g)
    f = thread.spawn(owned fn(x: int) => x)
