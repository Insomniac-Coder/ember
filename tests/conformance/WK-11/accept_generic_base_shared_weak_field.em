#$ test: run-pass
#$ rules: CLS-2, CLS-4, HEAP-3, HEAP-4, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, development, release
#$ stdout: 7
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# Generic base-class substitution must keep Weak[Shared[T]] field ownership
# through super.init, the derived layout, and the inherited field-drop chain.
struct Token:
    value: i32

[T]
open class Base:
    weak: Weak[Shared[Token]]
    marker: T

    fn init(weak: Weak[Shared[Token]], marker: T):
        this.weak = weak
        this.marker = marker

[T]
class Derived(Base[T]):
    flag: bool

    fn init(weak: Weak[Shared[Token]], marker: T, flag: bool):
        this.flag = flag
        super.init(weak, marker)

fn main():
    strong = Shared(Token(7))
    item = Derived[bool](Weak(strong), false, true)
    match item.weak.upgrade():
        Some(owner):
            println(owner.get().value)
        None:
            println(0)
