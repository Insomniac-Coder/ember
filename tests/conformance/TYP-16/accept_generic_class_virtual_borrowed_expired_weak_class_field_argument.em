#$ test: run-pass
#$ rules: TYP-16, CLS-2, CLS-4, OBJ-3, DSP-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, WK-14, TST-26
#$ profiles: debug, development, release
#$ stdout: 0
#$ assert-c: contains("static int32_t em_vt_7DerivedGda5cc12a188c7619b25d920e0657739a_slot0")
#$ assert-c: contains("->slot0")
#$ assert-c-count: contains("ember_weak_retain(") == 1
#$ assert-c-count: contains("ember_weak_release(") == 1
#$ assert-c: contains("ember_weak_upgrade(")

# A borrowed weakref Token class field crossing a generic virtual call remains a
# non-owning observer after its class target has expired. The derived override
# must take the None path without copying the field at dispatch.
class Token:
    value: i32

class Holder:
    weak: weakref Token

[T]
open class Base:
    marker: T

    fn init(marker: T):
        this.marker = marker

    virtual fn observe(weak: weakref Token) -> i32:
        if weak exists as owner:
            return owner.value
        else:
            return 0

[T]
class Derived(Base[T]):
    fn init(marker: T):
        super.init(marker)

    override fn observe(weak: weakref Token) -> i32:
        if weak exists as owner:
            return owner.value
        else:
            return 0

fn expired_holder() -> Holder:
    strong = Token(7)
    return Holder(weakref strong)

fn main():
    holder = expired_holder()
    derived = Derived[bool](true)
    base: Base[bool] = derived
    println(base.observe(holder.weak))
