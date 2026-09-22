#$ test: run-pass
#$ rules: TYP-16, CLS-2, CLS-4, DSP-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ assert-c: contains("static em_Weak_Shared_Token em_vt_Derived_bool_slot0")
#$ assert-c: contains("->slot0")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# A base-typed generic handle dispatches a virtual method that returns an
# inherited Weak[Shared[T]] field. The returned observer is an independent weak
# owner, so it remains usable after the virtual-call boundary and can upgrade
# the separate live Shared owner without copying the payload.
struct Token:
    value: i32

open class Base[T]:
    weak: Weak[Shared[Token]]
    marker: T

    fn init(mut self, weak: Weak[Shared[Token]], marker: T):
        self.weak = weak
        self.marker = marker

    virtual fn observer(self) -> Weak[Shared[Token]]:
        return self.weak

class Derived[T](Base[T]):
    fn init(mut self, weak: Weak[Shared[Token]], marker: T):
        super.init(weak, marker)

    override fn observer(self) -> Weak[Shared[Token]]:
        return self.weak

fn main():
    strong = Shared(Token(7))
    derived = Derived[bool](Weak(strong), true)
    base: Base[bool] = derived
    weak = base.observer()
    match weak.upgrade():
        Some(owner):
            println(owner.get().value)
        None:
            println(0)
