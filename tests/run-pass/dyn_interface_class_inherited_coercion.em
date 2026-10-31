#$ test: run-pass
#$ rules: TYP-22, IFC-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c: contains(int32_t (*slot0)(void*))
#$ assert-c: contains(int32_t (*slot1)(void*))

interface Parent:
    fn parent() -> i32

interface Child: needs Parent:
    fn child() -> i32

class Pair implements Parent, Child:
    left: i32
    right: i32

    fn parent() -> i32:
        return this.left

    fn child() -> i32:
        return this.right

fn total(value: ref dyn Child) -> i32:
    return value.parent() + value.child()

fn main():
    pair = Pair(20, 22)
    println(total(ref pair))
