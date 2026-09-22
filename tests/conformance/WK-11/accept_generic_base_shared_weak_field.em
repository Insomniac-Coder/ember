#$ test: run-pass
#$ rules: CLS-2, CLS-4, HEAP-3, HEAP-4, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# Generic base-class substitution must keep Weak[Shared[T]] field ownership
# through super.init, the derived layout, and the inherited field-drop chain.
struct Token:
    value: i32

open class Base[T]:
    weak: Weak[Shared[Token]]
    marker: T

    fn init(mut self, weak: Weak[Shared[Token]], marker: T):
        self.weak = weak
        self.marker = marker

class Derived[T](Base[T]):
    flag: bool

    fn init(mut self, weak: Weak[Shared[Token]], marker: T, flag: bool):
        super.init(weak, marker)
        self.flag = flag

fn main():
    strong = Shared(Token(7))
    item = Derived[bool](Weak(strong), false, true)
    match item.weak.upgrade():
        Some(owner):
            println(owner.get().value)
        None:
            println(0)
