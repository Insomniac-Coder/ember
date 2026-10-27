#$ test: run-pass
#$ rules: R8, R12
#$ stdout:
#$ start
#$ 2
## R8 (c), R12: the way out the refusal gives: the `for_each` that pushes to `LOG` runs after the loop
## over `LOG` (verdict 21).

static mut LOG: Array[String] = []

struct Score:
    name: String

fn main() changes LOG:
    scores = [Score(name=String("ann"))]
    LOG.push(String("start"))
    for line in LOG:
        println(line)
    scores.iter().for_each(fn(s) changes LOG: LOG.push(copy of s.name))
    println(LOG.len())
