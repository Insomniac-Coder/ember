#$ test: run-pass
#$ rules: BRW-18, CTL-2
#$ stdout: 1
#$ stdout: 3
## design rules: R12, R8
## Verdicts 21, 38 and 86, R8 (c), R12 (rules.md lines 703-711): `for_each` takes the lambda as a
## template parameter, and the call changes `LOG`, worked out from the lambda, with no word on it or on
## `main`; the walk of `LOG` ends first, then `for_each` runs, as the E3020 message of the nearest
## refused program says (reject_for_each_inside_a_loop_over_log).

static mut LOG: Array[int] = []

struct Score:
    points: int

fn main():
    LOG.push(1)
    scores = [Score(points=3), Score(points=5)]
    for entry in LOG:
        println(entry)
    scores.iter().for_each(fn(s): LOG.push(s.points))
    println(LOG.len())
