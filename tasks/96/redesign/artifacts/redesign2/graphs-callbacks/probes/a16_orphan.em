# A16: a program (nobody depends on it) gives another package's type another package's interface.
struct Vec3:
    x: f32

extend Array[Vec3] implements Neg:
    type Output = Array[Vec3]
    fn neg(self) -> Array[Vec3]:
        return []

fn main():
    v: Array[Vec3] = [Vec3(x=1.0)]
    println((-v).len())
