#$ test: run-pass
#$ rules: TYP-16, STR-1
#$ stdout: Tree.Leaf(value=3) true
#$ 4 Tree.Empty
# D-278 extended (D-304) — a generic enum's and a generic class's methods may
# name the type with its own parameters (`Tree[T]`, `Slot[T]`), as a generic
# struct's may: it is the instance. Their associated functions are called
# on an instance, `Tree[int].leaf(3)` and `Slot[int].make(4)`, including when
# that call is the first use of the instance. Both were E1010.

enum Tree[T]:
    Leaf(value: T)
    Empty

    fn leaf(v: T) -> Tree[T]:
        return Tree[T].Leaf(v)

    fn nothing() -> Tree[T]:
        return Tree[T].Empty

    fn same_shape(self, other: Tree[T]) -> bool:
        return true

class Slot[T]:
    value: T

    fn make(v: T) -> Slot[T]:
        return Slot[T](value=v)

fn main():
    t = Tree[int].leaf(3)
    println(t, t.same_shape(Tree[int].nothing()))
    s = Slot[int].make(4)
    println(s.value, Tree[str].nothing())
