#$ test: compile-fail
#$ rules: GRM-47, GRM-43
## `[GRM-47]` — `owns` is never an operand: `owns i * 7` is `(owns i) * 7`, which ends `i` before the
## multiplication; in an argument the message names the operator and gives the line to write.

fn main():
    m: Map[int, int] = Map()
    i = 3
    m.insert(owns i * 7, owns i)    #$ error[E0119]: `owns` here is an operand of `*`, which only reads `i`
                                    #$ help: write the product in brackets: `m.insert(owns (i * 7), owns i)`
    println(m.len())
