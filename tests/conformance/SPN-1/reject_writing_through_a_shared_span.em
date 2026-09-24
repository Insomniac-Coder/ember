#$ test: compile-fail
#$ rules: SPN-1, BRW-1
#$ error[E3021]: cannot write through a shared `Span`
#$ help: write to the container itself
# `[SPN-1]` — a `Span` is the shared view; `MutSpan` is the one that writes.
# A slice is a `Span`, so writing through it would mutate a borrowed
# parameter behind its caller's back.

fn poke(xs: Array[int]):
    xs[1..][0] = 42

fn main():
    xs = [1, 2, 3]
    poke(xs)
    println(xs)
