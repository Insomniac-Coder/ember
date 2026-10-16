#$ test: run-fail
#$ rules: RT-4, DIA-23
#$ profiles: release
#$ stdout: 3
#$ panics: index 5 is out of bounds for a length of 2
#$ stderr: index 5 is out of bounds for a length of 2
#$ stderr: backtrace:
#$ stderr: main at
# `[RT-4]` — a release build prints a backtrace too: its symbols are kept (in a PDB beside the
# program on Windows, in its own symbol table elsewhere), so the frames the C compiler did not
# inline are named, back to `main`.

fn inner(xs: Array[int], i: int) -> int:
    return xs[i]

fn outer(xs: Array[int], i: int) -> int:
    return inner(xs, i) + inner(xs, 0)

fn main():
    xs = [1, 2]
    println(outer(xs, 1))
    println(outer(xs, 5))
