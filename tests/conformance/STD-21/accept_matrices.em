#$ test: run-pass
#$ rules: STD-21, EXP-9
#$ profiles: debug, release, shipping
#$ stdout: -2.0 Vec2(x=3.0, y=7.0) Vec2(x=1.0, y=2.0) Vec2(x=3.0, y=4.0) Vec2(x=1.0, y=3.0)
#$ Mat2(x_axis=Vec2(x=1.0, y=0.0), y_axis=Vec2(x=0.0, y=1.0)) Mat2(x_axis=Vec2(x=-1.0, y=-3.0), y_axis=Vec2(x=-2.0, y=-4.0)) true true true true
#$ 24.0 Vec3(x=3.0, y=3.0, z=4.0) true 6.0 true Vec3(x=2.0, y=0.0, z=1.0)
#$ Vec3(x=3.0, y=4.0, z=5.0) Vec3(x=2.0, y=2.0, z=2.0) 8.0 true Vec3(x=1.0, y=1.0, z=1.0)
#$ Vec4(x=1.0, y=2.0, z=3.0, w=1.0) Vec4(x=0.0, y=0.0, z=0.0, w=1.0) Vec4(x=2.0, y=4.0, z=6.0, w=1.0)
#$ true true true true true
#$ 84.0 true true true
#$ Vec3(x=0.0, y=0.0, z=-5.0) Vec3(x=1.0, y=0.0, z=-5.0) Vec3(x=0.0, y=0.0, z=0.0) Vec3(x=1.0, y=1.0, z=1.0)
# ODR-043 — `std.math`'s matrices are column-major: `x_axis`, `y_axis`, … are
# the columns, `m * v` is `m.x_axis * v.x + m.y_axis * v.y + …`, and `m * n`
# applies `n` first. A transform's translation is its `w_axis`;
# `transform_point3` takes a point (`w` = 1, divided by the result's `w`),
# `transform_vector3` a direction (`w` = 0). A matrix with no inverse has
# `try_inverse` `None`. Values printed are exact; the one inverse with
# rounding in it is compared within a tolerance.

from std.math import Vec2, Vec3, Vec4, Mat2, Mat3, Mat4

## Whether every entry of `a - b` is within `1e-5`.
fn near(a: Mat4, b: Mat4) -> bool:
    d = a - b
    worst = d.x_axis.abs().max(d.y_axis.abs()).max(d.z_axis.abs()).max(d.w_axis.abs())
    return max(max(worst.x, worst.y), max(worst.z, worst.w)) < 0.00001

fn main():
    m2 = Mat2(Vec2(1, 3), Vec2(2, 4))
    println(m2.determinant(), m2 * Vec2(1, 1), m2.transpose().x_axis, m2.row(1), m2.col(0))
    println(m2 * m2.inverse(), -m2, m2 + m2 == m2 * 2.0, 2.0 * m2 == m2 * 2.0, m2 - m2 == Mat2.ZERO, Mat2.from_cols(Vec2(1, 3), Vec2(2, 4)) == m2)
    m3 = Mat3(Vec3(2, 0, 0), Vec3(0, 3, 0), Vec3(1, 0, 4))
    println(m3.determinant(), m3 * Vec3(1, 1, 1), m3 * m3.inverse() == Mat3.IDENTITY, Mat3.from_scale(Vec3(1, 2, 3)).determinant(), Mat3.from_diagonal(Vec3(1, 2, 3)) == Mat3.from_scale(Vec3(1, 2, 3)), m3.row(0))
    m4 = Mat4.from_translation(Vec3(1, 2, 3)) * Mat4.from_scale(Vec3(2, 2, 2))
    println(m4.transform_point3(Vec3(1, 1, 1)), m4.transform_vector3(Vec3(1, 1, 1)), m4.determinant(), m4 * m4.inverse() == Mat4.IDENTITY, m4.inverse().transform_point3(Vec3(3, 4, 5)))
    println(m4.col(3), m4.row(3), m4 * Vec4(0.5, 1, 1.5, 1))
    println(Mat4.ZERO.try_inverse() is None, Mat4.IDENTITY.try_inverse() is not None, Mat2.from_cols(Vec2(1, 2), Vec2(2, 4)).try_inverse() is None, Mat4.IDENTITY.transpose() == Mat4.IDENTITY, Mat4.from_diagonal(Vec4(2, 2, 2, 1)) == Mat4.from_scale(Vec3(2, 2, 2)))
    g = Mat4(Vec4(2, 1, 0, 0), Vec4(1, 3, 1, 0), Vec4(0, 1, 4, 1), Vec4(1, 0, 1, 5))
    println(g.determinant(), near(g * g.inverse(), Mat4.IDENTITY), near(g.inverse() * g, Mat4.IDENTITY), g.transpose().transpose() == g)
    # A camera at (0, 0, 5) looking at the origin, and one moved along `x`.
    view = Mat4.look_at_rh(Vec3(0, 0, 5), Vec3(0, 0, 0), Vec3(0, 1, 0))
    side = Mat4.look_at_rh(Vec3(-1, 0, 5), Vec3(-1, 0, 0), Vec3(0, 1, 0))
    ortho = Mat4.orthographic_rh(-2.0, 2.0, -2.0, 2.0, 0.0, 10.0)
    println(view.transform_point3(Vec3(0, 0, 0)), side.transform_point3(Vec3(0, 0, 0)), ortho.transform_point3(Vec3(0, 0, 0)), ortho.transform_point3(Vec3(2, 2, -10)))
