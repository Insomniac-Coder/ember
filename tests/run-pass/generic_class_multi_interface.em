#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1
#$ profiles: debug, development, release
#$ stdout: 42

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

fn total(value: ref dyn Child) -> i32:
    return value.parent() + value.child()

fn main():
    pair = Pair[bool](20, 22, false)
    println(total(ref pair))
