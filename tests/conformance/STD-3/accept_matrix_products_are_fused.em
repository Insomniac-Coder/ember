#$ test: run-pass
#$ rules: STD-3, STD-21
#$ profiles: debug, release, shipping
#$ stdout: Vec2(x=5.9604645e-08, y=0.0) Vec3(x=5.9604645e-08, y=0.0, z=0.0) Vec4(x=5.9604645e-08, y=0.0, z=0.0, w=0.0)
#$ Vec3(x=5.9604645e-08, y=1.0002441, z=0.0) Vec2(x=5.9604645e-08, y=0.0)
#$ 0.0 5.9604645e-08
# `[STD-3]` — `std.math`'s matrix products and transforms are fused
# multiply-adds: each term after the first is added with one rounding. With
# `e = 1 + 2^-12`, `e * e` is `1 + 2^-11 + 2^-24`, which `f32` rounds to
# `1 + 2^-11`; so `e * e - (1 + 2^-11)` is 0 in two roundings and `2^-24`
# fused. Each product below takes that term last.

from std.math import Vec2, Vec3, Vec4, Mat2, Mat3, Mat4

fn main():
    e: f32 = 1.0 + 1.0 / 4096.0
    c: f32 = -(1.0 + 1.0 / 2048.0)
    m2 = Mat2(Vec2(c, 0), Vec2(e, 0))
    m3 = Mat3(Vec3(c, 0, 0), Vec3(e, 0, 0), Vec3.ZERO)
    m4 = Mat4(Vec4(c, 0, 0, 0), Vec4.ZERO, Vec4.ZERO, Vec4(e, 0, 0, 0))
    println(m2 * Vec2(1, e), m3 * Vec3(1, e, 0), m4 * Vec4(1, 0, 0, e))
    # A matrix times a matrix is the products of its columns; a transformed
    # point is a product with `w` = 1.
    shear = Mat4(Vec4(1, 0, 0, 0), Vec4(e, 1, 0, 0), Vec4(0, 0, 1, 0), Vec4(0, 0, 0, 1))
    println(shear.transform_point3(Vec3(c, e, 0)), (m2 * Mat2(Vec2(1, e), Vec2.ZERO)).x_axis)
    # The same sum written out is two roundings.
    println(e * e + c, e.mul_add(e, c))
