#$ test: run-pass
#$ rules: TYP-22, IFC-1, HEAP-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_Child em_vt_dyn_Child_Signal)
#$ assert-c: contains(em_vt_dyn_Child_Signal_slot0)
#$ assert-c: contains(em_vt_dyn_Child_Signal_slot1)

interface Parent:
    fn parent(self) -> i32

interface Child: Parent:
    fn child(self) -> i32

enum Signal implements Parent, Child:
    Ready

    fn parent(self) -> i32:
        return 20

    fn child(self) -> i32:
        return 22

fn main():
    boxed: Box[dyn Child] = Box(Signal.Ready)
    println(boxed.parent() + boxed.child())
