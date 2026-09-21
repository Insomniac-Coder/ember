#$ test: run-pass
#$ rules: OBJ-2, DSP-3, IFC-3
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(em_vt_dyn_Child_Pixel)
#$ assert-c-count: contains(ember_itable_lookup) == 1

# A Child class handle exposes the parent-first Child table through its TypeInfo.
interface Parent:
    fn parent(self) -> i32

interface Child: Parent:
    fn child(self) -> i32

class Pixel implements Parent, Child:
    left: i32
    right: i32

    fn parent(self) -> i32:
        return self.left

    fn child(self) -> i32:
        return self.right

fn total(value: Child) -> i32:
    return value.parent() + value.child()

fn main():
    pixel = Pixel(20, 22)
    println(total(pixel))
