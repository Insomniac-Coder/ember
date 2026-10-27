#$ test: compile-fail
#$ rules: R12
#$ help: scores.iter().for_each(fn(s) changes LOG: LOG.push(s.points))
## Verdict 21, R12 (rules.md lines 543-549): `for_each` accepts a function that changes things if it says
## so; the same lambda without `changes LOG` is refused on its own first line (E2105). rules.md section
## 3's line 819 message, in this program's names.

static mut LOG: Array[int] = []

struct Score:
    points: int

fn main() changes LOG:
    scores = [Score(points=3), Score(points=5)]
    scores.iter().for_each(fn(s): LOG.push(s.points))    #$ error[E2105]: `for_each` runs this function, which pushes to `LOG`; say so on its first line: `scores.iter().for_each(fn(s) changes LOG: LOG.push(s.points))`
    println(LOG.len())
