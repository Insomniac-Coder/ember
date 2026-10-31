#$ test: run-pass
#$ rules: TYP-16, CLS-2, CLS-4, OBJ-3, DSP-1, OWN-2, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, WK-14, TST-26
#$ profiles: debug, development, release
#$ stdout: 7
#$ assert-c: contains("static int32_t em_vt_7DerivedGda5cc12a188c7619_slot0")
#$ assert-c: contains("->slot0")
#$ assert-c-count: contains("ember_weak_retain(") == 2
#$ assert-c: contains("ember_weak_release((ember_obj_header*)weak.value);")
#$ assert-c: contains("ember_weak_upgrade(")

# Passing Weak[Token] to an owned generic virtual parameter copies one weak
# observer for the derived override. The Holder keeps its field observer while
# the callee independently upgrades and then destroys its owned weak argument.
class Token:
    value: i32

class Holder:
    weak: Weak[Token]

[T]
open class Base:
    marker: T

    fn init(marker: T):
        this.marker = marker

    virtual fn observe(owns weak: Weak[Token]) -> i32:
        match weak.upgrade():
            Some(owner):
                return owner.value
            None:
                return 0

[T]
class Derived(Base[T]):
    fn init(marker: T):
        super.init(marker)

    override fn observe(owns weak: Weak[Token]) -> i32:
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
