#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, DRP-6, HEAP-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_Child em_vt_dyn_Child_Pixel_bool)

interface Parent:
    fn parent(self) -> i32:
        return 20

interface Child: Parent:
    fn child(self) -> i32:
        return 22

class Pixel[T] implements Parent, Child:
    marker: T

fn total(value: ref dyn Child) -> i32:
    return value.parent() + value.child()

fn main():
    pixel = Pixel[bool](false)
    println(total(ref pixel))
    boxed: Box[dyn Child] = Box(Pixel[bool](false))
    println(boxed.parent() + boxed.child())
