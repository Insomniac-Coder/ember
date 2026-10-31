#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c: contains(em_vt_dyn_6Parent_5Child_for_4Pair_slot0)
#$ assert-c: contains(em_vt_dyn_6Parent_5Child_for_4Pair_slot1)

interface Parent:
    fn parent() -> i32

interface Child: needs Parent:
    fn child() -> i32

struct Pair implements Parent, Child:
    left: i32
    right: i32

    fn parent() -> i32:
        return this.left

    fn child() -> i32:
        return this.right

fn total(value: ref dyn Parent + Child) -> i32:
    return value.parent() + value.child()

fn main():
    pair = Pair(20, 22)
    println(total(ref pair))
