#$ test: run-pass
#$ rules: EXP-1
#$ stdout: 400

## D-330: checking an expression recurses once for each level of it, and a
## debug build of the compiler spends about 65 KB of stack on a level of a
## binary operator. The compiler ran on the main thread, whose stack is 1 MB
## on Windows and 8 MB on Linux, so a twelve-deep polynomial compiled on
## Linux and overflowed on Windows. It runs on a thread of its own now, with
## the same stack on every host. This sum is four hundred levels deep: more
## than either main thread held.

fn main():
    x: i64 = 1
    total = (x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x
        + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x
        + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x
        + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x
        + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x
        + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x
        + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x
        + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x
        + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x
        + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x
        + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x
        + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x
        + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x
        + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x
        + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x
        + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x
        + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x
        + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x
        + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x
        + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x + x)
    println(total)
