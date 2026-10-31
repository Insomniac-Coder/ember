#$ test: run-pass
#$ rules: TYP-16, CLS-2, CLS-4, OBJ-3, DSP-1, FN-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, WK-14, TST-26
#$ profiles: debug, development, release
#$ stdout: 7
#$ assert-c: contains("static int32_t em_vt_7DerivedGda5cc12a188c7619_slot0")
#$ assert-c: contains("->slot0")
#$ assert-c-count: contains("ember_weak_retain(") == 1
#$ assert-c-count: contains("ember_weak_release(") == 1
#$ assert-c: contains("ember_weak_upgrade(")

# A Weak[Token] field passed to a generic virtual method uses the default
# borrowed parameter mode. The base-typed dispatch reuses the Holder field and
# must not copy an additional weak class owner at the virtual-call boundary.
class Token:
    value: i32

class Holder:
    weak: Weak[Token]

[T]
open class Base:
    marker: T

    fn init(marker: T):
        this.marker = marker

    virtual fn observe(weak: Weak[Token]) -> i32:
        match weak.upgrade():
            Some(owner):
                return owner.value
            None:
                return 0

[T]
class Derived(Base[T]):
    fn init(marker: T):
        super.init(marker)

    override fn observe(weak: Weak[Token]) -> i32:
        match weak.upgrade():
            Some(owner):
                return owner.value
            None:
                return 0

fn main():
    strong = Token(7)
    holder = Holder(Weak(strong))
    derived = Derived[bool](true)
    base: Base[bool] = derived
    println(base.observe(holder.weak))
