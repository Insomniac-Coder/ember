#$ test: run-pass
#$ rules: TYP-16, CLS-2, CLS-4, DSP-1, HEAP-3, HEAP-4, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, development, release
#$ stdout: 8
#$ assert-c: contains("static int32_t em_vt_7DerivedGda5cc12a188c7619b25d920e0657739a_slot0")
#$ assert-c: contains("->slot0")
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c: contains("ember_weak_release(")

# A base-typed handle to a generic derived class dispatches its virtual method
# through the derived slot. The inherited Weak[Shared[T]] field remains a normal
# weak observer across the upcast and is read only through Shared.get().
struct Token:
    value: i32

[T]
open class Base:
    weak: Weak[Shared[Token]]
    marker: T

    fn init(weak: Weak[Shared[Token]], marker: T):
        this.weak = weak
        this.marker = marker

    virtual fn value() -> i32:
        match this.weak.upgrade():
            Some(owner):
                return owner.get().value
            None:
                return 0

[T]
class Derived(Base[T]):
    fn init(weak: Weak[Shared[Token]], marker: T):
        super.init(weak, marker)

    override fn value() -> i32:
        match this.weak.upgrade():
            Some(owner):
                return owner.get().value + 1
            None:
                return 1

fn main():
    strong = Shared(Token(7))
    derived = Derived[bool](Weak(strong), true)
    base: Base[bool] = derived
    println(base.value())
