#$ test: run-pass
#$ rules: TYP-20, TYP-21
#$ stdout: 6.0 2
# D-413, ODR-097 — the package that declares `Vec3` may write `f32`'s
# `Mul[Vec3]`: `Vec3` is the first of the implementation's types it
# declares, and no type parameter stands alone before it. So may a
# program write a standard interface for its own type, and its own
# interface for a standard type.

struct Vec3:
    x: f32

extend f32 implements Mul[Vec3]:
    type Output = Vec3
    fn mul(self, v: Vec3) -> Vec3:
        return Vec3(x = self * v.x)

interface Size:
    fn size(self) -> int

extend bool implements Size:
    fn size(self) -> int:
        return 2

fn main():
    s: f32 = 2.0
    v = s * Vec3(x = 3.0)
    println(v.x, true.size())
