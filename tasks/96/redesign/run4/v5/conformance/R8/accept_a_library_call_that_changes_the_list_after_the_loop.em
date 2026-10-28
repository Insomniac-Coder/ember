#$ test: run-pass
#$ rules: BRW-16, CLO-1
#$ stdout:
#$ start
#$ 2
## design rules: R8, R12
## R8 (c), R12: the way out the refusal gives: the `for_each` whose function pushes to `LOG` runs
## after the loop over `LOG` (verdicts 21, 86: the lambda writes no word).

static mut LOG: Array[String] = []

struct Score:
    name: String

fn main():
    scores = [Score(name=String("ann"))]
    LOG.push(String("start"))
    for line in LOG:
        println(line)
    scores.iter().for_each(fn(s): LOG.push(copy of s.name))
    println(LOG.len())
