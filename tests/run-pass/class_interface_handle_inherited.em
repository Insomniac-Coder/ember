#$ test: run-pass
#$ rules: OBJ-2, DSP-3, IFC-3
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c: contains(em_vt_dyn_5Child_for_5Pixel)
#$ assert-c-count: contains(ember_itable_lookup) == 1

# A Child class handle exposes the parent-first Child table through its TypeInfo.
interface Parent:
    fn parent() -> i32

interface Child: needs Parent:
    fn child() -> i32

class Pixel implements Parent, Child:
    left: i32
    right: i32

    fn parent() -> i32:
        return this.left

    fn child() -> i32:
        return this.right

fn total(value: Child) -> i32:
    return value.parent() + value.child()

fn main():
    pixel = Pixel(20, 22)
    println(total(pixel))
