#$ test: run-pass
#$ rules: TYP-16, TYP-22, DRP-6, HEAP-1, IFC-1
#$ profiles: debug, development, release
#$ stdout: 20
#$ stdout: 22
#$ assert-c: contains(em_vt_dyn_4Left_5Right_for_4Pair_slot0)
#$ assert-c: contains(em_vt_dyn_4Left_5Right_for_4Pair_slot1)

interface Left:
    fn left() -> i32

interface Right:
    fn right() -> i32

struct Pair implements Left, Right:
    first: i32
    second: i32

    fn left() -> i32:
        return this.first

    fn right() -> i32:
        return this.second

fn main():
    boxed: owns dyn Left + Right = owns Pair(20, 22)
    println(boxed.left())
    println(boxed.right())
