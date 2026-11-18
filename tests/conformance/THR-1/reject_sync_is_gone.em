#$ test: compile-fail
#$ rules: THR-1, ATT-1
# `[THR-1]`, `[ATT-1]` — a class needs no mark to be shared between threads.

@sync    #$ error[E0104]: `@sync` is gone: a class needs no mark to be shared between threads; remove `@sync`
class Stats:
    hits: int = 0

fn main():
    println(Stats().hits)
