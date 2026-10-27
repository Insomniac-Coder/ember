#$ test: run-pass
#$ rules: R4, R5
#$ stdout: ann 3
## R4, R5, verdict 3: a `Mutex` is changed through a plain name by its own operation, under its lock,
## and the call takes no word. What `clear` changes is worked out from its body; no first line names it
## (verdict 86).

from std.sync import Mutex

fn top(scores: Mutex[Array[String]]) -> String:
    with g = scores.lock():
        return copy of g[0]

fn top_len(scores: Mutex[Array[String]]) -> int:
    with g = scores.lock():
        return g[0].len()

fn clear(scores: Mutex[Array[String]]):
    with g = scores.lock():
        g.clear()

fn main():
    m: Mutex[Array[String]] = Mutex([String("ann")])
    best = top(m)
    n = top_len(m)
    clear(m)
    println(best, n)
