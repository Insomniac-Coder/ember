#$ test: run-pass
#$ rules: TYP-16, IFC-1, IFC-3, TYP-22
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_Child em_vt_dyn_Child_Holder_bool)

interface Parent:
    fn parent(self) -> i32:
        return 20

interface Child: Parent:
    fn child(self) -> i32:
        return 22

class Holder[T]:
    marker: T

extend[T] Holder[T] implements Parent, Child:
    pass

fn answer(value: ref dyn Child) -> i32:
    return value.parent() + value.child()

fn main():
    holder = Holder[bool](false)
    println(answer(ref holder))
    boxed: Box[dyn Child] = Box(Holder[bool](false))
    println(boxed.parent() + boxed.child())
