#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(em_vt_dyn_multi__Parent__Child_Pair_slot0)
#$ assert-c: contains(em_vt_dyn_multi__Parent__Child_Pair_slot1)

interface Parent:
    fn parent(self) -> i32

interface Child: Parent:
    fn child(self) -> i32

struct Pair implements Parent, Child:
    left: i32
    right: i32

    fn parent(self) -> i32:
        return self.left

    fn child(self) -> i32:
        return self.right

fn total(value: ref dyn Parent + Child) -> i32:
    return value.parent() + value.child()

fn main():
    pair = Pair(20, 22)
    println(total(ref pair))
