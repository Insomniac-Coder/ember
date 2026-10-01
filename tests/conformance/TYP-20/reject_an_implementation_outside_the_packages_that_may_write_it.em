#$ test: compile-fail
#$ rules: TYP-20
# D-413, ODR-097 — an implementation is written in the package of its
# interface or of its type, or in the package of a type among the
# interface's arguments that comes before any parameter standing alone.
# Elsewhere it is `E2121`; it was accepted. A recipe is asked where it is
# declared, a blanket over its free parameter too: `T` stands alone before
# any type of this package in `Mul[T]`.

struct Vec3:
    x: f32

extend bool implements Add[bool]:    #$ error[E2121]: `bool` and `Add[bool]` are both another package's, so this package cannot implement one for the other
    type Output = bool
    fn add(self, other: bool) -> bool:
        return self or other

extend Array[Vec3] implements Neg:    #$ error[E2121]: `Array[Vec3]` and `Neg` are both another package's, so this package cannot implement one for the other
    type Output = Array[Vec3]
    fn neg(self) -> Array[Vec3]:
        return []

extend[T] Option[T] implements Neg:    #$ error[E2121]: `Option[T]` and `Neg` are both another package's, so this package cannot implement one for the other
    type Output = Option[T]
    fn neg(self) -> Option[T]:
        return None

extend[T: Copy] f32 implements Mul[T]:    #$ error[E2121]: `f32` and `Mul[T]` are both another package's, so this package cannot implement one for the other
    type Output = f32
    fn mul(self, other: T) -> f32:
        return self

fn main():
    println(1)
