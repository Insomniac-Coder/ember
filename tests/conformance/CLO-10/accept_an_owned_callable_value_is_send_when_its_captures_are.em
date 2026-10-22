#$ test: run-pass
#$ rules: CLO-10, THR-8, THR-10, TYP-17
#$ profiles: debug, development, release
#$ stdout: 12 11 21
# `[CLO-10]`, `[THR-8]` (ODR-218) — an owned callable value is `Send` when everything it captures
# is: a named `owned fn` capturing a number and a `String` moves to a thread, and so does one a
# generic function holds as `F: fn() -> int + Send` (a type parameter is what its bounds require),
# captured by the thread's own function. A function captures nothing, so it always is.

import std.thread

fn run_later[F: fn() -> int + Send](owned task: F) -> int:
    h = thread.spawn(owned fn() -> int => task())
    return h.join()

fn twenty_one() -> int:
    return 21

fn main():
    n = 5
    label = "seven".to_string()
    both = owned fn() -> int => n + label.len() + 2
    h = thread.spawn(both)
    println(h.join(), run_later(owned fn() -> int => n + 6), run_later(twenty_one))
