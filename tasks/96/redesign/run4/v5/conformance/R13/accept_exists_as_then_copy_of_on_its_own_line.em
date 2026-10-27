#$ test: run-pass
#$ rules: R13, R3
#$ stdout: 4 3
## R13, R3: the two-line form is valid too (verdict 66): `exists as v` names what the map holds, read
## only, and `mine = copy of v` makes a number of your own, which may change; `copy of` works on every
## `Copy` type (verdict 68).

fn main():
    scores: Map[String, int] = Map()
    scores[String("ann")] = 3
    if scores.get("ann") exists as v:
        mine = copy of v
        mine += 1
        println(mine, v)
