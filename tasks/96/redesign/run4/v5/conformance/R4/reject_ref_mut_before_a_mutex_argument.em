#$ test: compile-fail
#$ rules: R4, R5
#$ help: best = top(m)
## R4, verdict 3: an argument given to a `Mutex` parameter takes no word at the call; its own
## "only me" is the lock's block (AN-30).

from std.sync import Mutex

fn top(scores: Mutex[Array[String]]) -> String:
    with g = scores.lock():
        return copy of g[0]

fn main():
    m: Mutex[Array[String]] = Mutex([String("ann")])
    best = top(ref mut m)    #$ error[E3028]: `top` takes the lock of `scores` itself; a `Mutex` takes no word at the call: `top(m)`
    println(best)
