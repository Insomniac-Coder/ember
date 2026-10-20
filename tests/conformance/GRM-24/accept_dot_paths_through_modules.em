#$ test: run-pass
#$ rules: GRM-24, MOD-3, STR-7
#$ profiles: debug, development, release
#$ stdout: 12
#$ 12
#$ 2
#$ 5 5
# `.` is the one path separator. A path resolves left to right: a module
# reached through another module's namespace, then an item in it, and a
# type's constant through the type (`[STR-7]`).
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
    println(paths.inner.Tile.SIDE, inner.Tile.SIDE)
