#$ test: run-pass
#$ rules: CLO-3, FN-1
#$ stdout: [5, 0]
# D-598 — a lambda that gives a captured view to a `mut` parameter writes through it, though no
# `ref mut` is written: the closure borrows it mutably. It was `E3023` on the closure's environment.

fn add_to(mut s: MutSpan[int], n: int):
    s[0] += n

fn main():
    counts = [0, 0]
    first, _second = counts.as_mut_span().split_at(1)
    f = fn() => add_to(first, 5)
    f()
    println(counts)
