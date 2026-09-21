#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1
#$ profiles: debug, release, shipping
#$ stdout: 20
#$ stdout: 22
#$ assert-c: contains(static const struct em_vt_dyn_multi__Left__Right em_vt_dyn_multi__Left__Right_Pair)

interface Left:
    fn left(self) -> i32

interface Right:
    fn right(self) -> i32

struct Pair implements Left, Right:
    first: i32
    second: i32

    fn left(self) -> i32:
        return self.first

    fn right(self) -> i32:
        return self.second

fn print_both(value: ref dyn Left + Right):
    println(value.left())
    println(value.right())

fn main():
    pair = Pair(20, 22)
    print_both(ref pair)
