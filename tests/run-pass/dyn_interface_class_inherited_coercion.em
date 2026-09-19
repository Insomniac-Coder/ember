#$ test: run-pass
#$ rules: TYP-22, IFC-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(int32_t (*slot0)(void*))
#$ assert-c: contains(int32_t (*slot1)(void*))

interface Parent:
    fn parent(self) -> i32

interface Child: Parent:
    fn child(self) -> i32

class Pair implements Parent, Child:
    left: i32
    right: i32

    fn parent(self) -> i32:
        return self.left

    fn child(self) -> i32:
        return self.right

fn total(value: ref dyn Child) -> i32:
    return value.parent() + value.child()

fn main():
    pair = Pair(20, 22)
    println(total(ref pair))
