#$ test: compile-fail
#$ rules: R8, R12
#$ help: move line 17 below the loop
## R8 (c), R12: a library call given a function counts as making that function's changes (verdicts 21,
## 86: the lambda writes no word); inside a loop over `LOG` a `for_each` whose function pushes to `LOG`
## is refused. The message of rules section 3, on `scores.iter()` (`copy of s.name`: `s` only reads).

static mut LOG: Array[String] = []

struct Score:
    name: String

fn main():
    scores = [Score(name=String("ann"))]
    LOG.push(String("start"))
    for line in LOG:
        scores.iter().for_each(fn(s): LOG.push(copy of s.name))    #$ error[E3020]: `scores.iter().for_each(...)` (line 17) pushes to `LOG` (through the function it is given) while this loop walks `LOG` (line 16); call it after the loop: move line 17 below the loop
        println(line)
