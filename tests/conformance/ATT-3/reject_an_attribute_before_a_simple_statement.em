#$ test: compile-fail
#$ rules: ATT-3
# `[ATT-3]` — a statement attribute attaches to the next compound statement, never to a simple one
# (`E0108`).

fn main():
    c = 1
    @simd                           #$ error[E0108]: `@simd` must precede a compound statement
    c = c + 1
    println(c)
