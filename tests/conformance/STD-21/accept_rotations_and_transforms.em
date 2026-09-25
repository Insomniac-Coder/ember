#$ test: run-pass
#$ rules: STD-21
#$ profiles: debug, release, shipping
#$ stdout: true true true true true
#$ true true true true
#$ Vec3(x=1.0, y=2.0, z=3.0) true Quat(x=0.0, y=0.0, z=0.0, w=1.0) Quat(x=-1.0, y=-2.0, z=-3.0, w=4.0) 5.477226
#$ true true true true
#$ true true true true true
#$ Vec3(x=12.0, y=2.0, z=2.0) Vec3(x=2.0, y=2.0, z=2.0) Vec3(x=34.0, y=0.0, z=0.0) Vec3(x=12.0, y=2.0, z=2.0)
#$ Vec3(x=1.0, y=2.0, z=3.0) Vec3(x=1.0, y=2.0, z=3.0) Vec3(x=1.0, y=2.0, z=3.0) true true
# ODR-043 — rotations are right-handed: a quarter turn about `z` takes `x` to
# `y`, about `x` takes `y` to `z`, about `y` takes `z` to `x`. `q * p`
# rotates by `p`, then by `q`; a `Transform` scales, then rotates, then
# moves, and `t * u` applies `u` first. A matrix built from a rotation turns
# the same way. Anything computed through `sin` or `cos` is compared within a
# tolerance, not printed: a platform's last digits are not the language's.

from std.math import Vec2, Vec3, Mat2, Mat3, Mat4, Quat, Transform, PI

fn close(a: Vec3, b: Vec3) -> bool:
    return (a - b).length() < 0.000001

fn close2(a: Vec2, b: Vec2) -> bool:
    return (a - b).length() < 0.000001

fn main():
    quarter: f32 = PI / 2.0
    q = Quat.from_rotation_z(quarter)
    println(close(q * Vec3(1, 0, 0), Vec3(0, 1, 0)), close(Quat.from_rotation_x(quarter) * Vec3(0, 1, 0), Vec3(0, 0, 1)), close(Quat.from_rotation_y(quarter) * Vec3(0, 0, 1), Vec3(1, 0, 0)), close(q * q * Vec3(1, 0, 0), Vec3(-1, 0, 0)), close(Quat.from_axis_angle(Vec3(0, 0, 1), quarter) * Vec3(0, 1, 0), Vec3(-1, 0, 0)))
    # `x` then `z`: a quarter turn about `x` leaves `x` alone, and the one
    # about `z` takes it to `y`.
    xz = q * Quat.from_rotation_x(quarter)
    println(close(xz * Vec3(1, 0, 0), Vec3(0, 1, 0)), close(xz * Vec3(0, 1, 0), Vec3(0, 0, 1)), abs((q * q.conjugate()).w - 1.0) < 0.000001, abs(q.length() - 1.0) < 0.000001)
    println(Quat.IDENTITY * Vec3(1, 2, 3), Quat.IDENTITY * Quat.IDENTITY == Quat.IDENTITY, Quat(0, 0, 0, 2).normalize(), Quat(1, 2, 3, 4).conjugate(), Quat(1, 2, 3, 4).length())
    half = Quat.IDENTITY.slerp(q, 0.5)
    println(close(half * Vec3(1, 0, 0), Vec3(0.70710677, 0.70710677, 0)), abs(Quat.IDENTITY.slerp(q, 1.0).dot(q) - 1.0) < 0.000001, abs(Quat.IDENTITY.slerp(q, 0.0).dot(Quat.IDENTITY) - 1.0) < 0.000001, abs(q.inverse().dot(q.conjugate()) - 1.0) < 0.000001)
    println(close(Mat3.from_quat(q) * Vec3(1, 0, 0), Vec3(0, 1, 0)), close(Mat4.from_quat(q).transform_vector3(Vec3(1, 0, 0)), Vec3(0, 1, 0)), close(Mat4.from_rotation_z(quarter).transform_point3(Vec3(1, 0, 0)), Vec3(0, 1, 0)), close2(Mat2.from_angle(quarter) * Vec2(1, 0), Vec2(0, 1)), close(Mat4.from_scale_rotation_translation(Vec3(2, 2, 2), q, Vec3(10, 0, 0)).transform_point3(Vec3(1, 0, 0)), Vec3(10, 2, 0)))
    t = Transform(Vec3(10, 0, 0), Quat.IDENTITY, Vec3(2, 2, 2))
    println(t.transform_point(Vec3(1, 1, 1)), t.transform_vector(Vec3(1, 1, 1)), (t * t).transform_point(Vec3(1, 0, 0)), t.to_mat4().transform_point3(Vec3(1, 1, 1)))
    println(Transform.IDENTITY.transform_point(Vec3(1, 2, 3)), Transform.from_translation(Vec3(1, 2, 3)).transform_point(Vec3.ZERO), Transform.from_scale(Vec3(1, 2, 3)).transform_vector(Vec3.ONE), close(Transform.from_rotation(q).transform_point(Vec3(1, 0, 0)), Vec3(0, 1, 0)), close((Transform.from_translation(Vec3(0, 0, 5)) * Transform.from_rotation(q)).transform_point(Vec3(1, 0, 0)), Vec3(0, 1, 5)))
