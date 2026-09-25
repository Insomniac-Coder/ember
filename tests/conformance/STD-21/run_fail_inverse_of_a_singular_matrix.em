#$ test: run-fail
#$ rules: STD-21, PAN-1
#$ panics: inverse of a singular matrix; use try_inverse
#$ stdout: true
#$ profiles: debug, release, shipping
# ODR-043 — a matrix whose determinant is zero has no inverse: `inverse`
# panics as `normalize` of a zero vector does, and `try_inverse` is `None`.

from std.math import Vec3, Mat3

fn main():
    flat = Mat3(Vec3(1, 2, 3), Vec3(2, 4, 6), Vec3(0, 0, 1))
    println(flat.try_inverse() is None)
    println(flat.inverse())
