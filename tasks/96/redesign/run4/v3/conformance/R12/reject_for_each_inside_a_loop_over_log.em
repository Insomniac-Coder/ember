#$ test: compile-fail
#$ rules: R12, R8
#$ help: scores.iter().for_each(fn(s) changes LOG: LOG.push(s.points))
## Verdict 21, R8 (c), R12 (rules.md lines 546-548): a library call given a function that changes `LOG`
## is a change of `LOG`, refused inside a loop over `LOG` (E3020). rules.md section 3's line 837 message,
## in this program's lines.

static mut LOG: Array[int] = []

struct Score:
    points: int

fn main() changes LOG:
    LOG.push(1)
    scores = [Score(points=3), Score(points=5)]
    for entry in LOG:
        println(entry)
        scores.iter().for_each(fn(s) changes LOG: LOG.push(s.points))    #$ error[E3020]: this loop walks `LOG` (line 16), and `scores.iter().for_each(...)` (line 18) changes `LOG` (the function it is given says `changes LOG`); write `scores.iter().for_each(fn(s) changes LOG: LOG.push(s.points))` after the loop
