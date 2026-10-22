#$ test: run-pass
#$ rules: STD-21, STD-14, ERR-13
#$ profiles: debug, development, release
#$ stdout: Some(Vec2(x=3.0, y=4.0)) None Some(Vec2(x=2.0, y=4.0)) None
#$ stdout: Some(Vec3(x=0.0, y=0.0, z=1.0)) None Some(Vec3(x=0.0, y=1.0, z=0.0)) None
#$ stdout: Some(Vec4(x=5.0, y=6.0, z=7.0, w=1.0)) None Some(Vec4(x=0.0, y=0.0, z=1.0, w=7.0)) None
# `[STD-14]`, `[ERR-13]` (ODR-254) — a matrix's `col` and `row` are accessors, and a column or row
# past the matrix is the caller's bug: each panics naming its twin, `try_col` and `try_row`, which
# give `None` there and the column or row otherwise.

from std.math import Vec2, Vec3, Vec4, Mat2, Mat3, Mat4

fn main():
    m2 = Mat2(Vec2(1, 2), Vec2(3, 4))
    println(m2.try_col(1), m2.try_col(2), m2.try_row(1), m2.try_row(-1))
    m3 = Mat3.IDENTITY
    println(m3.try_col(2), m3.try_col(3), m3.try_row(1), m3.try_row(-1))
    m4 = Mat4.from_translation(Vec3(5, 6, 7))
    println(m4.try_col(3), m4.try_col(4), m4.try_row(2), m4.try_row(9))
