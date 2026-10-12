#$ test: run-pass
#$ rules: GRM-11, PHIL-14
#$ stdout: {1}
## `[GRM-11]` — there are no block expressions: `{ 1 }` is a set literal, as
## in Python.

fn main():
    x = {
        1
    }
    println(x)
