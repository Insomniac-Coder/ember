#$ test: compile-fail
#$ rules: R5, R12
#$ help: fn clear(scores: Mutex[Array[String]]) changes Mutex[Array[String]]:
## R5, verdict 29: a change made under a lock is still named on the first line, by the type of what
## changes (AN-30, E2105).

from std.sync import Mutex

fn clear(scores: Mutex[Array[String]]):
    with g = scores.lock():
        g.clear()    #$ error[E2105]: `clear` changes the list in `scores` under its lock (`g.clear()`, line 11); say so on its first line: `fn clear(scores: Mutex[Array[String]]) changes Mutex[Array[String]]:`

fn main():
    m: Mutex[Array[String]] = Mutex([String("ann")])
    clear(m)
    println("cleared")
