# A16: print a list of my vectors; sum any numbers; a special case for one type.

struct Vec3:
    x: f32
    y: f32
    z: f32

extend Array[Vec3] implements Neg:
    type Output = Array[Vec3]
    fn neg(self) -> Array[Vec3]:
        return []

fn sum[T](xs: Span[T]) -> T:
    acc = xs[0]
    for x in xs[1..]:
        acc = acc + x
    return acc

interface Describe:
    fn describe(self) -> String

extend[T: Debug] T implements Describe:
    fn describe(self) -> String:
        return f"{self:?}"

extend i32 implements Describe:
    fn describe(self) -> String:
        return f"int {self}"

fn main():
    println(sum([1, 2, 3]))
