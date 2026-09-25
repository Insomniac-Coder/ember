#$ test: run-pass
#$ rules: STD-21, EXP-9
#$ profiles: debug, release, shipping
#$ stdout: Vec3(x=1.0, y=1.0, z=1.0) Vec3(x=1.0, y=1.0, z=1.0) true true false true false
#$ Aabb(min=Vec3(x=-1.0, y=0.0, z=0.0), max=Vec3(x=2.0, y=2.0, z=4.0)) Aabb(min=Vec3(x=0.0, y=0.0, z=0.0), max=Vec3(x=4.0, y=4.0, z=4.0)) true
#$ true false true false true
#$ Vec3(x=-3.0, y=1.0, z=1.0) Some(5.0) Some(4.0) Some(8.0)
#$ None None None None
#$ Some(0.0) Some(0.0) Some(0.0) Some(3.0)
#$ 3.0 -3.0 Plane(normal=Vec3(x=0.0, y=0.0, z=1.0), d=-2.0) Plane(normal=Vec3(x=0.0, y=1.0, z=0.0), d=-1.0)
#$ true false false false true false false
#$ true false true false
#$ true false
# ODR-043 — `std.math`'s shapes. A box holds the points between its corners,
# its boundary included; a ball those within its radius. A ray's
# intersections are the least `t ≥ 0` at which it is on the plane or in the
# ball or box, 0 when it starts there, `None` when there is none. A frustum
# is the six planes of a projection times a view, each facing in; here a
# 90° right-handed perspective looking down `-z` from the origin, near 1 and
# far 100. Every value here is exact: the tests do not depend on the last
# digits of a platform's `tan`.

from std.math import Vec3, Mat4, Aabb, Sphere, Ray, Plane, Frustum, PI

fn main():
    b = Aabb(Vec3(0, 0, 0), Vec3(2, 2, 2))
    println(b.center(), b.half_extents(), Aabb.from_center_half_extents(Vec3.ONE, Vec3.ONE) == b, b.contains(Vec3(1, 1, 1)), b.contains(Vec3(3, 1, 1)), b.intersects(Aabb(Vec3(1, 1, 1), Vec3(5, 5, 5))), b.intersects(Aabb(Vec3(2.5, 0, 0), Vec3(3, 1, 1))))
    println(b.expand(Vec3(-1, 0, 4)), b.union(Aabb(Vec3(3, 3, 3), Vec3(4, 4, 4))), b.contains(Vec3(2, 2, 2)))
    s = Sphere(Vec3(0, 0, 0), 1.0)
    println(s.contains(Vec3(0.5, 0.5, 0.5)), s.intersects(Sphere(Vec3(3, 0, 0), 1.5)), s.intersects_aabb(b), s.intersects_aabb(Aabb(Vec3(1, 1, 1), Vec3(2, 2, 2))), s.intersects(Sphere(Vec3(3, 0, 0), 2.0)))
    ray = Ray(Vec3(-5, 1, 1), Vec3(1, 0, 0))
    wall = Plane.from_point_normal(Vec3(3, 0, 0), Vec3(-1, 0, 0))
    println(ray.at(2.0), ray.intersect_aabb(b), ray.intersect_sphere(Sphere(Vec3(0, 1, 1), 1.0)), ray.intersect_plane(wall))
    # Missed, or behind the ray.
    away = Ray(Vec3(-5, 1, 1), Vec3(-1, 0, 0))
    println(Ray(Vec3(-5, 9, 1), Vec3(1, 0, 0)).intersect_aabb(b), away.intersect_aabb(b), away.intersect_sphere(Sphere(Vec3(0, 1, 1), 1.0)), away.intersect_plane(wall))
    # From inside, or on the plane.
    inside = Ray(Vec3(1, 1, 1), Vec3(1, 0, 0))
    println(inside.intersect_aabb(b), inside.intersect_sphere(Sphere(Vec3(1, 1, 1), 1.0)), Ray(Vec3(3, 5, 5), Vec3(0, 1, 0)).intersect_plane(wall), Ray(Vec3(0, 0, 0), Vec3(2, 0, 0)).intersect_plane(Plane(Vec3(1, 0, 0), -6.0)))
    floor = Plane.from_points(Vec3(0, 0, 0), Vec3(1, 0, 0), Vec3(0, 1, 0))
    println(floor.signed_distance(Vec3(0, 0, 3)), floor.signed_distance(Vec3(5, 5, -3)), Plane.from_point_normal(Vec3(0, 0, 2), Vec3(0, 0, 5)), Plane(Vec3(0, 2, 0), -2.0).normalize())
    proj = Mat4.perspective_rh(PI / 2.0, 1.0, 1.0, 100.0)
    view = Mat4.look_at_rh(Vec3(0, 0, 0), Vec3(0, 0, -1), Vec3(0, 1, 0))
    f = Frustum.from_view_projection(proj * view)
    println(f.contains_point(Vec3(0, 0, -10)), f.contains_point(Vec3(0, 0, 10)), f.contains_point(Vec3(0, 0, -200)), f.contains_point(Vec3(0, 0, -0.5)), f.contains_point(Vec3(5, 5, -10)), f.contains_point(Vec3(15, 0, -10)), f.contains_point(Vec3(0, -15, -10)))
    println(f.intersects_sphere(Sphere(Vec3(0, 0, -0.5), 0.6)), f.intersects_sphere(Sphere(Vec3(0, 0, 5), 1.0)), f.intersects_aabb(Aabb(Vec3(-1, -1, -3), Vec3(1, 1, -2))), f.intersects_aabb(Aabb(Vec3(-50, -1, -20), Vec3(-30, 1, -10))))
    ortho = Frustum.from_view_projection(Mat4.orthographic_rh(-1.0, 1.0, -1.0, 1.0, 0.0, 10.0))
    println(ortho.contains_point(Vec3(0.5, -0.5, -5)), ortho.contains_point(Vec3(2, 0, -5)))
