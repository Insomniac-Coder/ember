#$ test: run-fail
#$ rules: STD-21, STD-14, ERR-13
#$ panics: a Mat3 has columns 0 to 2 only; use .try_col(i) for an Option
#$ stdout: Vec3(x=0.0, y=1.0, z=0.0)
#$ profiles: debug, development, release
# `[ERR-13]` (ODR-254) — `col` of a column the matrix does not have is the caller's bug: it panics,
# and the message names the twin that gives an `Option`.

from std.math import Mat3

fn main():
    m = Mat3.IDENTITY
    println(m.col(1))
    println(m.col(3))
