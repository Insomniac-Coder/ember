#$ test: compile-fail
#$ rules: BRW-19
#$ help: return a copy, or do the work inside the `with` block
## design rules: R8
## R8 (e), row 4: a view made through a lock's guard points into the guard and does not leave its
## `with` block; row 4's message, word for word. AN-30, point 6.

from std.sync import Mutex

fn top(scores: Mutex[Array[String]]) -> ref String:
    with g = scores.lock():
        return ref g[0]    #$ error[E3063]: this points into the locked list, which can change once the lock ends; return a copy, or do the work inside the `with` block

fn main():
    m: Mutex[Array[String]] = Mutex([String("ann")])
    println(top(m))
