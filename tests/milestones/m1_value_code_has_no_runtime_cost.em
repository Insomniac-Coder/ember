#$ test: run-pass
#$ rules: STR-1, TYP-11, LEX-16, PHIL-2
struct Vec3:
    x: f32
    y: f32
    z: f32

fn add(a: Vec3, b: Vec3) -> Vec3:
    return Vec3(a.x + b.x, a.y + b.y, a.z + b.z)

fn main():
    c = add(Vec3(1, 2, 3), Vec3(4, 5, 6))
    println(c.x)
#$ stdout: 5
#$ assert-c: !contains("ember_alloc")
