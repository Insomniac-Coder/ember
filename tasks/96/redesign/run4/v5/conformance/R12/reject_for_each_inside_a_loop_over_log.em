#$ test: compile-fail
#$ rules: BRW-18, CTL-2
#$ help: move line 19 below the loop
## design rules: R12, R8
## Verdicts 21, 38 and 86, R8 (c), R12 (rules.md lines 703-711): "The call counts as making what the
## function given changes: `scores.for_each(fn(s): LOG.push(copy of s.name))` changes `LOG`, so inside a
## loop over `LOG` it is refused (E3020)". rules.md section 3's line 1066 first message, in this program's
## lines. The nearest allowed program is accept_for_each_after_the_loop_over_log.

static mut LOG: Array[int] = []

struct Score:
    points: int

fn main():
    LOG.push(1)
    scores = [Score(points=3), Score(points=5)]
    for entry in LOG:
        println(entry)
        scores.iter().for_each(fn(s): LOG.push(s.points))    #$ error[E3020]: `scores.iter().for_each(...)` (line 19) pushes to `LOG` (through the function it is given) while this loop walks `LOG` (line 17); call it after the loop: move line 19 below the loop
