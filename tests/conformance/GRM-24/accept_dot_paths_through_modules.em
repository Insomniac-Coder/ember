#$ test: run-pass
#$ rules: GRM-24, MOD-3
#$ profiles: debug, release, shipping
#$ stdout: 12
#$ 12
#$ 2
# `.` is the one path separator. A path resolves left to right: a module
# reached through another module's namespace, then an item in it.
import paths
import paths.inner as inner

fn main():
    println(paths.inner.triple(4))
    println(inner.triple(4))
    s = inner.Shape.Square(2)
    match s:
        inner.Shape.Square(side):
            println(side)
        _:
            println(0)
