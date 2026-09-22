#$ test: run-pass
#$ rules: TYP-16, CLS-2, CLS-4, DSP-1, HEAP-3, HEAP-4, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 8
#$ assert-c: contains("static int32_t em_vt_Derived_bool_slot0")
#$ assert-c: contains("->slot0")
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c: contains("ember_weak_release(")

# A base-typed handle to a generic derived class dispatches its virtual method
# through the derived slot. The inherited Weak[Shared[T]] field remains a normal
# weak observer across the upcast and is read only through Shared.get().
struct Token:
    value: i32

open class Base[T]:
    weak: Weak[Shared[Token]]
    marker: T

    fn init(mut self, weak: Weak[Shared[Token]], marker: T):
        self.weak = weak
        self.marker = marker

    virtual fn value(self) -> i32:
        match self.weak.upgrade():
            Some(owner):
                return owner.get().value
            None:
                return 0

class Derived[T](Base[T]):
    fn init(mut self, weak: Weak[Shared[Token]], marker: T):
        super.init(weak, marker)

    override fn value(self) -> i32:
        match self.weak.upgrade():
            Some(owner):
                return owner.get().value + 1
            None:
                return 1

fn main():
    strong = Shared(Token(7))
    derived = Derived[bool](Weak(strong), true)
    base: Base[bool] = derived
    println(base.value())
