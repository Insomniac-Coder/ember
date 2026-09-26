#$ test: run-pass
#$ rules: TYP-16, CLS-2, CLS-4, DSP-1, EXC-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, WK-14, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ assert-c: contains("static em_Weak_Shared_Token em_vt_Derived_bool_slot0")
#$ assert-c: contains("->slot0")
#$ assert-c: contains("ember_object_begin_write")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# A mutable generic virtual call updates an inherited counter and returns its
# Weak[Shared[Token]] field. The base-typed receiver closes its write interval
# at the call while the copied observer remains independently upgradeable.
struct Token:
    value: i32

open class Base[T]:
    weak: Weak[Shared[Token]]
    marker: T
    count: i32

    fn init(mut self, weak: Weak[Shared[Token]], marker: T, count: i32):
        self.weak = weak
        self.marker = marker
        self.count = count

    virtual fn observer(mut self) -> Weak[Shared[Token]]:
        self.count = self.count + 1
        return self.weak

class Derived[T](Base[T]):
    fn init(mut self, weak: Weak[Shared[Token]], marker: T, count: i32):
        super.init(weak, marker, count)

    override fn observer(mut self) -> Weak[Shared[Token]]:
        self.count = self.count + 2
        return self.weak

fn main():
    strong = Shared(Token(7))
    derived = Derived[bool](Weak(strong), true, 0)
    base: Base[bool] = derived
    weak = base.observer()
    match weak.upgrade():
        Some(owner):
            println(owner.get().value)
        None:
            println(0)
