#$ test: run-pass
#$ rules: THR-4, THR-5
#$ profiles: debug
#$ stdout: 3 3
#$ stderr: warning: lock order inversion
#$ stderr: while holding the one it took at
# `[THR-4]` — in the debug profile the runtime records the order each thread takes locks in, and
# when one thread takes two in the order opposite to another's it prints `warning: lock order
# inversion` with both sites; the program runs on unchanged. (One after the other here, so the
# test cannot deadlock.)

from std.sync import Mutex
import std.thread

fn both(first: ref Mutex[int], second: ref Mutex[int]) -> int:
    with x = first.lock():
        with y = second.lock():
            return x + y

fn main():
    a = Mutex(1)
    b = Mutex(2)
    first = 0
    with scope = thread.scope():
        h = scope.spawn(fn() => both(a, b))
        first = h.join()
    second = both(b, a)
    println(first, second)
