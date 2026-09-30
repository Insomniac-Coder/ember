#$ test: run-pass
#$ rules: TYP-19, TYP-20
#$ stdout: copy text 1 2 array option
# ODR-092 — implementations whose types can never be one type are each kept:
# a bound separates two where the type it bounds is written out in full and
# does not meet it (`String` is not `Copy`), and different instances,
# different shapes and different interface arguments never meet.

interface Named:
    fn name(self) -> str

interface Kind:
    fn kind(self) -> str

struct Pair[A, B]:
    a: A
    b: B

struct Wrap[T]:
    x: T

extend[T: Copy] Pair[T, int] implements Named:
    fn name(self) -> str:
        return "copy"

extend[U] Pair[String, U] implements Named:
    fn name(self) -> str:
        return "text"

struct V:
    x: int

extend V implements Add[int]:
    type Output = int
    fn add(self, other: int) -> int:
        return self.x + other

extend V implements Add[V]:
    type Output = int
    fn add(self, other: V) -> int:
        return self.x + other.x + 1

extend[T] Wrap[Array[T]] implements Kind:
    fn kind(self) -> str:
        return "array"

extend[T] Wrap[Option[T]] implements Kind:
    fn kind(self) -> str:
        return "option"

fn main():
    p = Pair(a = 5, b = 1)
    q = Pair(a = String.from("s"), b = 1)
    println(p.name(), q.name(), V(x = 0) + 1, V(x = 0) + V(x = 1), Wrap(x = [1]).kind(), Wrap(x = Some(1)).kind())
