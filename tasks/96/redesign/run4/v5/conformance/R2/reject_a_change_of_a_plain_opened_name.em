#$ test: compile-fail
#$ rules: BRW-14, BRW-15
#$ help: for a number of your own write `if scores.get("ann") exists as copy of v:`, or write `mine = copy of v` and `mine += 1` in place of line 12, and `println(mine)` in place of line 13
## design rules: R2, R13
## R2, R13: a name opened with `exists as` stands for what the opening gave and keeps it for its whole
## scope: a change through a plain opened name is refused (E3023), and `copy of` after `exists as`
## gives a value of your own, which may change (verdicts 66, 67).

fn main():
    scores: Map[String, int] = Map()
    scores[String("ann")] = 3
    if scores.get("ann") exists as v:
        v += 1                 #$ error[E3023]: `v` is the number `scores.get("ann")` holds and stays it for the whole block
        println(v)
