#$ test: run-pass
#$ rules: STR-1
## "Every struct has a synthesised **memberwise constructor**
## `Name(field0, field1, …)` accepting positional or named arguments; fields
## with defaults may be omitted."

struct Vec3:
    x: f32
    y: f32
    z: f32 = 0.0

fn main():
    a = Vec3(1.0, 2.0, 3.0)
    b = Vec3(x=1.0, y=2.0)
    println(a.z)
    println(b.z)
#$ stdout: 3
#$ 0
