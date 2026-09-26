#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, DRP-6, EXC-1, HEAP-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 1
#$ stdout: 8
#$ assert-c: contains("ember_object_begin_write")
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c: !contains("ember_box_new_copy")

# A generic class payload in Box[dyn Read + Bump] keeps one multi-interface
# dynamic carrier. Its mut slot updates the original class field, while the
# shared slot upgrades a Weak[Shared[T]] field through Shared.get().
interface Read:
    fn read(self) -> i32

interface Bump:
    fn bump(mut self) -> i32

struct Token:
    value: i32

class Counter[T] implements Read, Bump:
    weak: Weak[Shared[Token]]
    marker: T
    count: i32

    fn read(self) -> i32:
        match self.weak.upgrade():
            Some(owner):
                return owner.get().value + self.count
            None:
                return self.count

    fn bump(mut self) -> i32:
        self.count = self.count + 1
        return self.count

fn main():
    strong = Shared(Token(7))
    boxed: Box[dyn Read + Bump] = Box(Counter[bool](Weak(strong), true, 0))
    println(boxed.bump())
    println(boxed.read())
