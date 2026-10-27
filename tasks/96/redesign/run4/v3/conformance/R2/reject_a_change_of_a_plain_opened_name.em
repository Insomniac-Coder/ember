#$ test: compile-fail
#$ rules: R2, R13
#$ help: for a number of your own write `if scores.get("ann") exists as copy of v:`
## R2, R13: a name opened with `exists as` stands for what the opening gave and keeps it for its whole
## scope: a change through a plain opened name is refused (E3023), and `copy of` after `exists as`
## gives a value of your own.

fn main():
    scores: Map[String, int] = Map()
    scores[String("ann")] = 3
    if scores.get("ann") exists as v:
        v += 1                 #$ error[E3023]: `v` is the number `scores.get("ann")` holds and stays it for the whole block
        println(v)
