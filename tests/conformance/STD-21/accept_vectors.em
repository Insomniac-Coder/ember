#$ test: run-pass
#$ rules: STD-21, TYP-21, STR-1
#$ profiles: debug, release, shipping
#$ stdout: Vec3(x=3.0, y=4.0, z=5.0) Vec3(x=-1.0, y=0.0, z=1.0) Vec3(x=2.0, y=4.0, z=6.0) Vec3(x=0.5, y=1.0, z=1.5) Vec3(x=-1.0, y=-2.0, z=-3.0)
#$ Vec3(x=2.0, y=4.0, z=6.0) Vec3(x=2.0, y=4.0, z=6.0) Vec3(x=0.5, y=1.0, z=1.5) Vec3(x=2.0, y=3.0, z=4.0) Vec3(x=1.0, y=0.0, z=-1.0)
#$ 12.0 Vec3(x=-2.0, y=4.0, z=-2.0) 14.0 5.0 1.4142135 2.0
#$ Vec3(x=0.6, y=0.8, z=0.0) Vec3(x=0.0, y=0.0, z=0.0) Vec3(x=1.5, y=2.0, z=2.5) Vec3(x=1.0, y=2.0, z=2.0) Vec3(x=2.0, y=2.0, z=3.0) Vec3(x=1.0, y=2.0, z=3.0)
#$ Vec3(x=3.0, y=3.0, z=0.0) Vec4(x=1.0, y=2.0, z=3.0, w=4.0) Vec3(x=1.0, y=2.0, z=3.0) Vec3(x=1.0, y=2.0, z=3.0)
#$ Vec2(x=0.0, y=0.0) Vec2(x=1.0, y=1.0) Vec4(x=0.0, y=0.0, z=0.0, w=1.0) Vec2(x=5.0, y=5.0) 5.0 Vec2(x=0.0, y=1.0)
#$ Vec3(x=3.0, y=5.0, z=7.0) Vec3(x=1.0, y=2.0, z=3.0) true false
#$ IVec3(x=3, y=-4, z=1) IVec3(x=1, y=1, z=0) IVec3(x=21, y=-21, z=6) IVec3(x=21, y=-21, z=6) IVec3(x=-7, y=7, z=-2) 2 IVec3(x=7, y=7, z=2)
#$ IVec2(x=-3, y=4) IVec4(x=1, y=2, z=3, w=0) IVec3(x=-1, y=-1, z=-1) IVec2(x=-1, y=4)
#$ UVec2(x=6, y=10) UVec2(x=10, y=18) UVec2(x=2, y=2) UVec2(x=1, y=1) UVec3(x=5, y=9, z=0) UVec2(x=5, y=4)
# ODR-043 — `std.math`'s vectors. The operators work component by component,
# a scalar on either side acts on every component (`2.0 * a` is `f32`'s
# `Mul[Vec3]`, `[TYP-21]`), and `op=` goes through the operator. The integer
# vectors have floor division and modulo; the unsigned ones have no `-v` and
# no `abs`. Each is a struct with public components, so its memberwise
# constructor is public (`[STR-1]`), and an untyped literal argument takes the
# component type (`Vec3(1, 2, 3)`).

from std.math import Vec2, Vec3, Vec4, IVec2, IVec3, IVec4, UVec2, UVec3

fn main():
    a = Vec3(1, 2, 3)
    b = Vec3.splat(2.0)
    println(a + b, a - b, a * b, a / b, -a)
    println(a * 2.0, 2.0 * a, a / 2.0, 1.0 + a, 2.0 - a)
    println(a.dot(b), a.cross(b), a.length_squared(), Vec3(3, 4, 0).length(), a.distance(b), a.distance_squared(b))
    println(Vec3(3, 4, 0).normalize(), Vec3.ZERO.normalize_or_zero(), a.lerp(b, 0.5), a.min(b), a.max(b), (-a).abs())
    v = Vec3.X
    v += Vec3.Y
    v *= 3.0
    println(v, a.extend(4.0), Vec4(1, 2, 3, 4).truncate(), Vec2(1, 2).extend(3.0))
    p = Vec2.ONE
    p *= 5.0
    println(Vec2.ZERO, Vec2.ONE, Vec4.W, p, Vec2(3, 4).length(), Vec2.Y)
    # `self * a + b`, one rounding for each component (`[STD-3]`).
    println(a.mul_add(b, Vec3.ONE), a.mul_add(Vec3.ONE, Vec3.ZERO), a == Vec3(1, 2, 3), a == b)
    i = IVec3(7, -7, 2)
    println(i // 2, i % 2, i * 3, 3 * i, -i, i.dot(IVec3.ONE), i.abs())
    j = IVec2(3, -4)
    println(-j, IVec3(1, 2, 3).extend(0), IVec3.ONE * -1, j.min(IVec2(-1, 9)).max(IVec2(-1, 4)))
    u = UVec2(5, 9)
    println(u + UVec2.ONE, u * 2, u // UVec2(2, 4), u % 2, u.extend(0), u.min(UVec2(7, 4)))
