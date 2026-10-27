#$ test: run-pass
#$ rules: R12, R8
#$ stdout: 1
#$ stdout: 3
## R8 (c), R12 (rules.md lines 662-664): the walk of `LOG` ends first, then `for_each` changes it, as the
## E3020 message of the nearest refused program says.

static mut LOG: Array[int] = []

struct Score:
    points: int

fn main() changes LOG:
    LOG.push(1)
    scores = [Score(points=3), Score(points=5)]
    for entry in LOG:
        println(entry)
    scores.iter().for_each(fn(s) changes LOG: LOG.push(s.points))
    println(LOG.len())
