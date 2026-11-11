#$ test: run-pass
#$ rules: TYP-16, IFC-1, IFC-3, TYP-22
#$ profiles: debug, development, release
#$ stdout: 42
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_5Child em_vt_dyn_5Child_for_6HolderGda5cc12a188c7619b25d920e0657739a)

interface Parent:
    fn parent() -> i32:
        return 20

interface Child: needs Parent:
    fn child() -> i32:
        return 22

[T]
class Holder:
    marker: T

extend[T] Holder[T] implements Parent, Child:
    pass

fn answer(value: ref dyn Child) -> i32:
    return value.parent() + value.child()

fn main():
    holder = Holder[bool](false)
    println(answer(ref holder))
    boxed: owns dyn Child = owns Holder[bool](false)
    println(boxed.parent() + boxed.child())
