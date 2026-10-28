#$ test: compile-fail
#$ rules: R8
#$ help: return a copy, or do the work inside the `with` block
## R8 (e): the same for `RwLock`'s `read()`: a view of the read guard does not leave its `with` block.
## AN-30, point 6.

from std.sync import RwLock

fn top(scores: RwLock[Array[String]]) -> ref String:
    with g = scores.read():
        return ref g[0]    #$ error[E3063]: this points into the read-locked list, which can change once the lock ends; return a copy, or do the work inside the `with` block

fn main():
    m: RwLock[Array[String]] = RwLock([String("ann")])
    println(top(m))
