#$ test: compile-fail
#$ rules: FN-2a
#$ help: say so at the call: write `owns` before it, in brackets: `m.insert(owns (i * 7), owns i)`
# `[FN-2a]` — for a computed argument the message says "this argument", and the help says to write
# `owns` before it, in brackets.

fn main():
    m: Map[int, int] = {}
    i = 3
    m.insert(i * 7, owns i)    #$ error[E3028]: `insert` takes this argument over (its first line says `owns k: K`)
