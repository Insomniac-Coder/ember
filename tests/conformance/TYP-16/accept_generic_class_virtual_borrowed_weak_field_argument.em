#$ test: run-pass
#$ rules: TYP-16, CLS-2, CLS-4, DSP-1, FN-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, development, release
#$ stdout: 7
#$ assert-c: contains("static int32_t em_vt_7DerivedGda5cc12a188c7619b25d920e0657739a_slot0")
#$ assert-c: contains("->slot0")
#$ assert-c-count: contains("ember_weak_retain(") == 1
#$ assert-c-count: contains("ember_weak_release(") == 1
#$ assert-c: contains("ember_weak_upgrade(")

# A weakref T field passed to a generic virtual method uses the default
# borrowed parameter mode. Dispatch must reuse the Holder field rather than
# copy a second weak owner across the virtual-call boundary.
struct Token:
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

fn main():
    strong = shares mut Token(7)
    holder = Holder(weakref strong)
    derived = Derived[bool](true)
    base: Base[bool] = derived
    println(base.observe(holder.weak))
