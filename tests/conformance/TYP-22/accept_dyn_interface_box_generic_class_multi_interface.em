#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, DRP-6, HEAP-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(em_vt_dyn_Child_Pair_bool)
#$ assert-c: !contains(ember_box_new_copy)

interface Parent:
    fn parent(self) -> i32

interface Child: Parent:
    fn child(self) -> i32

class Pair[T] implements Parent, Child:
    left: i32
    right: i32
    marker: T

    fn parent(self) -> i32:
        return self.left

    fn child(self) -> i32:
        return self.right

fn main():
    boxed: Box[dyn Child] = Box(Pair[bool](20, 22, false))
    println(boxed.parent() + boxed.child())
