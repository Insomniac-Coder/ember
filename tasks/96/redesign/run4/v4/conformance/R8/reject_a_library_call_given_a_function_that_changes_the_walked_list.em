#$ test: compile-fail
#$ rules: R8, R12
#$ help: scores.iter().for_each(fn(s) changes LOG: LOG.push(copy of s.name))
## R8 (c), R12: a library call given a function counts as making that function's changes (verdict 21);
## inside a loop over `LOG` a `for_each` whose function pushes to `LOG` is refused. The message of
## `V4/rules.md` section 3 (`copy of s.name`: `s` only reads), on `scores.iter()`.

static mut LOG: Array[String] = []

struct Score:
    name: String

fn main() changes LOG:
    scores = [Score(name=String("ann"))]
    LOG.push(String("start"))
    for line in LOG:
        scores.iter().for_each(fn(s) changes LOG: LOG.push(copy of s.name))    #$ error[E3020]: this loop walks `LOG` (line 16), and `scores.iter().for_each(...)` (line 17) changes `LOG` (the function it is given says `changes LOG`); write `scores.iter().for_each(fn(s) changes LOG: LOG.push(copy of s.name))` after the loop
        println(line)
