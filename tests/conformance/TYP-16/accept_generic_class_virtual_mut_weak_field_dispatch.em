#$ test: run-pass
#$ rules: TYP-16, CLS-2, CLS-4, DSP-1, BRW-1, EXC-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, development, release
#$ stdout: 9
#$ assert-c: contains("static int32_t em_vt_7DerivedGda5cc12a188c7619_slot0")
#$ assert-c: contains("->slot0")
#$ assert-c: contains("ember_object_begin_write")
#$ assert-c: contains("ember_weak_upgrade(")

# A base-typed generic handle dispatches a mut virtual call to the derived
# implementation. The override updates the inherited count and observes the
# inherited Weak[Shared[T]] field through the explicit Shared.get() boundary.
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

    virtual fn tick() -> i32:
        this.count = this.count + 1
        return this.count

[T]
class Derived(Base[T]):
    fn init(weak: Weak[Shared[Token]], marker: T, count: i32):
        super.init(weak, marker, count)

    override fn tick() -> i32:
        this.count = this.count + 2
        match this.weak.upgrade():
            Some(owner):
                return owner.get().value + this.count
            None:
                return this.count

fn main():
    strong = Shared(Token(7))
    derived = Derived[bool](Weak(strong), true, 0)
    base: Base[bool] = derived
    println(base.tick())
