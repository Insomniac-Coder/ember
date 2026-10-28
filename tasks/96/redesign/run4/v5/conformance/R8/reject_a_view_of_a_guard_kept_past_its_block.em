#$ test: compile-fail
#$ rules: BRW-19
#$ help: println(best)
## design rules: R8, R2
## R8 (e): a view of a guard stored in a name made before the `with` block, and used after the block,
## would outlive the guard. AN-30, point 6 (its message, with the way out that compiles here, R18).

from std.sync import Mutex

fn main():
    m: Mutex[Array[String]] = Mutex([String("ann")])
    fallback = String("none")
    best = ref fallback
    with g = m.lock():
        best = ref g[0]    #$ error[E3063]: `best` points into the locked list, which can change once the lock ends with the `with` block, and `best` is used after it on line 15; use it inside the block: write `println(best)` below line 14 in place of line 15
    println(best)
