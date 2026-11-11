#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, DRP-6, HEAP-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_5Child em_vt_dyn_5Child_for_5PixelGda5cc12a188c7619b25d920e0657739a)

interface Parent:
    fn parent() -> i32:
        return 20

interface Child: needs Parent:
    fn child() -> i32:
        return 22

[T]
class Pixel implements Parent, Child:
    marker: T

fn total(value: ref dyn Child) -> i32:
    return value.parent() + value.child()

fn main():
    pixel = Pixel[bool](false)
    println(total(ref pixel))
    boxed: Box[dyn Child] = Box(Pixel[bool](false))
    println(boxed.parent() + boxed.child())
