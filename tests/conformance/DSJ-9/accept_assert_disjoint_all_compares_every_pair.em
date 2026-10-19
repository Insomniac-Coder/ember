#$ test: run-pass
#$ rules: DSJ-9
#$ profiles: debug, development, release
#$ stdout: apart [1, 2] [3, 4] [5, 6]
#$ overlap [1, 2] [3, 4] [4, 5, 6]

# `[DSJ-9]` — `assert_disjoint_all` compares two to eight views pairwise.

from std.mem import assert_disjoint_all

fn main():
    x: Array[int] = [1, 2, 3, 4, 5, 6]
    match assert_disjoint_all(x[0..2], x[2..4], x[4..6]):
        Ok((a, b, c)):
            println("apart", a, b, c)
        Err((a, b, c)):
            println("overlap", a, b, c)
    match assert_disjoint_all(x[0..2], x[2..4], x[3..6]):
        Ok((a, b, c)):
            println("apart", a, b, c)
        Err((a, b, c)):
            println("overlap", a, b, c)
