#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-2, CLS-4, BRW-1, EXC-1, OWN-2, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, development, release
#$ stdout: 8
#$ assert-c: contains("em_vt_dyn_7Observe_4Bump_for_6HolderGda5cc12a188c7619_slot0")
#$ assert-c: contains("em_vt_dyn_7Observe_4Bump_for_6HolderGda5cc12a188c7619_slot1")
#$ assert-c: contains("ember_object_begin_write")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# A generic derived class with inherited state crosses a ref mut composed
# carrier with an owned weak argument. The mutable Observe slot updates the
# inherited counter and the Bump slot observes that same payload.
interface Observe:
    fn observe(owns weak: Weak[Shared[Token]]) -> i32

interface Bump:
    fn bump() -> i32

struct Token:
    value: i32

[T]
open class Base:
    marker: T
    count: i32

    fn init(marker: T):
        this.marker = marker
        this.count = 0

[T]
class Holder(Base[T]) implements Observe, Bump:
    fn init(marker: T):
        super.init(marker)

    fn observe(owns weak: Weak[Shared[Token]]) -> i32:
        this.count = this.count + 1
        match weak.upgrade():
            Some(owner):
                return owner.get().value
            None:
                return 0

    fn bump() -> i32:
        return this.count

fn inspect(ref mut value: dyn Observe + Bump, weak: Weak[Shared[Token]]) -> i32:
    return value.observe(owns weak) + value.bump()

fn main():
    strong = Shared(Token(7))
    holder = Holder[bool](true)
    println(inspect(ref mut holder, Weak(strong)))
