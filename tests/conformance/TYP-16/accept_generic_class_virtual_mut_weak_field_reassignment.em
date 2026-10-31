#$ test: run-pass
#$ rules: TYP-16, CLS-2, CLS-4, DSP-1, BRW-1, EXC-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, development, release
#$ stdout: 8
#$ assert-c: contains("static int32_t em_vt_7DerivedGda5cc12a188c7619_slot0")
#$ assert-c: contains("->slot0")
#$ assert-c: contains("ember_object_begin_write")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# A base-typed generic handle dispatches a mut virtual call that replaces its
# inherited Weak[Shared[T]] field. The override owns the new observer, drops
# the previous empty observer, and upgrades only the replacement.
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

    virtual fn replace(replacement: Weak[Shared[Token]]) -> i32:
        this.weak = replacement
        this.count = this.count + 1
        return this.count

[T]
class Derived(Base[T]):
    fn init(weak: Weak[Shared[Token]], marker: T, count: i32):
        super.init(weak, marker, count)

    override fn replace(replacement: Weak[Shared[Token]]) -> i32:
        this.weak = replacement
        this.count = this.count + 1
        match this.weak.upgrade():
            Some(owner):
                return owner.get().value + this.count
            None:
                return this.count

fn main():
    strong = Shared(Token(7))
    empty: Weak[Shared[Token]] = Weak[Shared[Token]].empty()
    derived = Derived[bool](empty, true, 0)
    base: Base[bool] = derived
    println(base.replace(Weak(strong)))
