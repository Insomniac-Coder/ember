#$ test: run-pass
#$ rules: R12, R5
#$ stdout: 2
## Verdicts 21 and 38, R12 (rules.md lines 659-668): `[C, F: fn(Item) changes C]` above
## `fn for_each(owns f: F) owns this changes C:` makes the call a change of `LOG`, which `main` names.

static mut LOG: Array[int] = []

struct Score:
    points: int

fn main() changes LOG:
    scores = [Score(points=3), Score(points=5)]
    scores.iter().for_each(fn(s) changes LOG: LOG.push(s.points))
    println(LOG.len())
