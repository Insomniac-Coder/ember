#$ test: run-pass
#$ rules: R8, R4, R5
#$ stdout: ann 3
## R8 (e): what leaves the `with` block is a copy of your own (`copy of g[0]`) or a number; `clear(m)`
## cannot end `best`. AN-30's NEW program, with `main` naming what `clear` changes: a change a callee's
## first line names by a type counts for its caller, its own local included (R5).

from std.sync import Mutex

fn top(scores: Mutex[Array[String]]) -> String:
    with g = scores.lock():
        return copy of g[0]

fn top_len(scores: Mutex[Array[String]]) -> int:
    with g = scores.lock():
        return g[0].len()

fn clear(scores: Mutex[Array[String]]) changes Mutex[Array[String]]:
    with g = scores.lock():
        g.clear()

fn main() changes Mutex[Array[String]]:
    m: Mutex[Array[String]] = Mutex([String("ann")])
    best = top(m)
    n = top_len(m)
    clear(m)
    println(best, n)
