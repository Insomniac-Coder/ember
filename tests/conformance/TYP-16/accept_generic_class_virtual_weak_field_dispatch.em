#$ test: run-pass
#$ rules: TYP-16, CLS-2, CLS-4, DSP-1, HEAP-3, HEAP-4, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, development, release
#$ stdout: 8
#$ assert-c: contains("static int32_t em_vt_7DerivedGda5cc12a188c7619b25d920e0657739a_slot0")
#$ assert-c: contains("->slot0")
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c: contains("ember_weak_release(")

# A base-typed handle to a generic derived class dispatches its virtual method
# through the derived slot. The inherited weakref T field remains a normal
# weak observer across the upcast and is read only through the owner read as its value.
struct Token:
    value: i32

[T]
open class Base:
    weak: weakref Token
    marker: T

    fn init(weak: weakref Token, marker: T):
        this.weak = weakref weak
        this.marker = marker

    virtual fn value() -> i32:
        if this.weak exists as owner:
            return owner.value
        else:
            return 0

[T]
class Derived(Base[T]):
    fn init(weak: weakref Token, marker: T):
        super.init(weak, marker)

    override fn value() -> i32:
        if this.weak exists as owner:
            return owner.value + 1
        else:
            return 1

fn main():
    strong = shares mut Token(7)
    derived = Derived[bool](weakref strong, true)
    base: Base[bool] = derived
    println(base.value())
