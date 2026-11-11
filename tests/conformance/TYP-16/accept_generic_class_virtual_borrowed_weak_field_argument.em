#$ test: run-pass
#$ rules: TYP-16, CLS-2, CLS-4, DSP-1, FN-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, development, release
#$ stdout: 7
#$ assert-c: contains("static int32_t em_vt_7DerivedGda5cc12a188c7619b25d920e0657739a_slot0")
#$ assert-c: contains("->slot0")
#$ assert-c-count: contains("ember_weak_retain(") == 1
#$ assert-c-count: contains("ember_weak_release(") == 1
#$ assert-c: contains("ember_weak_upgrade(")

# A Weak[Shared[T]] field passed to a generic virtual method uses the default
# borrowed parameter mode. Dispatch must reuse the Holder field rather than
# copy a second weak owner across the virtual-call boundary.
struct Token:
    value: i32

class Holder:
    weak: Weak[Shared[Token]]

[T]
open class Base:
    marker: T

    fn init(marker: T):
        this.marker = marker

    virtual fn observe(weak: Weak[Shared[Token]]) -> i32:
        match weak.upgrade():
            Some(owner):
                return owner.get().value
            None:
                return 0

[T]
class Derived(Base[T]):
    fn init(marker: T):
        super.init(marker)

    override fn observe(weak: Weak[Shared[Token]]) -> i32:
        match weak.upgrade():
            Some(owner):
                return owner.get().value
            None:
                return 0

fn main():
    strong = Shared(Token(7))
    holder = Holder(Weak(strong))
    derived = Derived[bool](true)
    base: Base[bool] = derived
    println(base.observe(holder.weak))
