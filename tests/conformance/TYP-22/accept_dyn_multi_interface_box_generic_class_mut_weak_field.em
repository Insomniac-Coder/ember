#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, DRP-6, EXC-1, HEAP-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, development, release
#$ stdout: 1
#$ stdout: 8
#$ assert-c: contains("ember_object_begin_write")
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c: !contains("ember_box_new_copy")

# A generic class payload in Box[dyn Read + Bump] keeps one multi-interface
# dynamic carrier. Its mut slot updates the original class field, while the
# shared slot upgrades a Weak[Shared[T]] field through Shared.get().
interface Read:
    fn read() -> i32

interface Bump:
    fn bump() -> i32

struct Token:
    value: i32

[T]
class Counter implements Read, Bump:
    weak: Weak[Shared[Token]]
    marker: T
    count: i32

    fn read() -> i32:
        match this.weak.upgrade():
            Some(owner):
                return owner.get().value + this.count
            None:
                return this.count

    fn bump() -> i32:
        this.count = this.count + 1
        return this.count

fn main():
    strong = Shared(Token(7))
    boxed: owns dyn Read + Bump = owns Counter[bool](Weak(strong), true, 0)
    println(boxed.bump())
    println(boxed.read())
