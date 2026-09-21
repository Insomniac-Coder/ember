#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, DRP-6, HEAP-1
#$ profiles: debug, release, shipping
#$ stdout: 20
#$ stdout: 22
#$ assert-c: contains(em_vt_dyn_multi__Left__Right_Pair)
#$ assert-c: !contains(ember_box_new_copy)

interface Left:
    fn left(self) -> i32

interface Right:
    fn right(self) -> i32

class Pair implements Left, Right:
    first: i32
    second: i32

    fn left(self) -> i32:
        return self.first

    fn right(self) -> i32:
        return self.second

fn main():
    boxed: Box[dyn Left + Right] = Box(Pair(20, 22))
    println(boxed.left())
    println(boxed.right())
