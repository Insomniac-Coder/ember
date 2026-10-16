#$ test: run-pass
#$ rules: THR-10, LT-3
#$ profiles: debug, release
#$ stdout: 5 hello world
# `[THR-10]` — a thread's function may capture a view with the static region: a string literal's
# (`[LT-3]`), directly or through a local it was bound to.

import std.thread

fn main():
    greeting = "hello"
    h = thread.spawn(owned fn() => greeting.len())
    g = thread.spawn(owned fn() => f"{greeting} world")
    println(h.join(), g.join())
