#$ test: compile-fail
#$ rules: THR-10, LT-42
#$ error[E3063]: `owned fn` captures `Span[i64]` by value, but that view is not static
# `[THR-10]` — a thread's function carries only the static region: a view of a local, which the
# thread could outlive, is refused where the `owned fn` is made (`[LT-42]`).

import std.thread

fn main():
    xs = [1, 2, 3]
    view: Span[int] = xs
    h = thread.spawn(owned fn() => view.len())
    println(h.join())
