#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, DRP-6, HEAP-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c: contains(em_vt_dyn_5Child_for_4PairGda5cc12a188c7619b25d920e0657739a)
#$ assert-c: !contains(ember_box_new_copy)

interface Parent:
    fn parent() -> i32

interface Child: needs Parent:
    fn child() -> i32

[T]
class Pair implements Parent, Child:
    left: i32
    right: i32
    marker: T

    fn parent() -> i32:
        return this.left

    fn child() -> i32:
        return this.right

fn main():
    boxed: owns dyn Child = owns Pair[bool](20, 22, false)
    println(boxed.parent() + boxed.child())
