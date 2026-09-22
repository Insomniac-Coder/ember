#$ test: run-pass
#$ rules: TYP-16, CLS-2, CLS-4, OBJ-3, DSP-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, WK-14, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 0
#$ assert-c: contains("static int32_t em_vt_Derived_bool_slot0")
#$ assert-c: contains("->slot0")
#$ assert-c-count: contains("ember_weak_retain(") == 1
#$ assert-c-count: contains("ember_weak_release(") == 1
#$ assert-c: contains("ember_weak_upgrade(")

# A borrowed Weak[Token] class field crossing a generic virtual call remains a
# non-owning observer after its class target has expired. The derived override
# must take the None path without copying the field at dispatch.
class Token:
    value: i32

class Holder:
    weak: Weak[Token]

open class Base[T]:
    marker: T

    fn init(mut self, marker: T):
        self.marker = marker

    virtual fn observe(self, weak: Weak[Token]) -> i32:
        match weak.upgrade():
            Some(owner):
                return owner.value
            None:
                return 0

class Derived[T](Base[T]):
    fn init(mut self, marker: T):
        super.init(marker)

    override fn observe(self, weak: Weak[Token]) -> i32:
        match weak.upgrade():
            Some(owner):
                return owner.value
            None:
                return 0

fn expired_holder() -> Holder:
    strong = Token(7)
    return Holder(Weak(strong))

fn main():
    holder = expired_holder()
    derived = Derived[bool](true)
    base: Base[bool] = derived
    println(base.observe(holder.weak))
