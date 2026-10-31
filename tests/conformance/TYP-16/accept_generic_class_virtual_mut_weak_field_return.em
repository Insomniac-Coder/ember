#$ test: run-pass
#$ rules: TYP-16, CLS-2, CLS-4, DSP-1, EXC-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, WK-14, TST-26
#$ profiles: debug, development, release
#$ stdout: 7
#$ assert-c: contains("static em_3std4core4WeakGa3404dfcc5f190c5 em_vt_7DerivedGda5cc12a188c7619_slot0")
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

[T]
open class Base:
    weak: Weak[Shared[Token]]
    marker: T
    count: i32

    fn init(weak: Weak[Shared[Token]], marker: T, count: i32):
        this.weak = weak
        this.marker = marker
        this.count = count

    virtual fn observer() -> Weak[Shared[Token]]:
        this.count = this.count + 1
        return this.weak

[T]
class Derived(Base[T]):
    fn init(weak: Weak[Shared[Token]], marker: T, count: i32):
        super.init(weak, marker, count)

    override fn observer() -> Weak[Shared[Token]]:
        this.count = this.count + 2
        return this.weak

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
