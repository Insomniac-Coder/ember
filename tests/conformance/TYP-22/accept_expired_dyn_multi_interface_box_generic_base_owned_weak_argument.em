#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-2, CLS-4, DRP-6, OWN-2, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, development, release
#$ stdout: 0
#$ stdout: 1
#$ assert-c: contains("em_vt_dyn_7Observe_4Bump_for_6HolderGda5cc12a188c7619b25d920e0657739a_slot0")
#$ assert-c: contains("em_vt_dyn_7Observe_4Bump_for_6HolderGda5cc12a188c7619b25d920e0657739a_slot1")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c: !contains("ember_box_new_copy")

# An owned weak argument through a composed dynamic box with generic-base state
# receives and destroys an independent observer after expiry. The inherited
# counter remains callable through the second interface slot.
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
        match weak.upgrade():
            Some(owner):
                return owner.get().value
            None:
                return 0

    fn bump() -> i32:
        this.count = this.count + 1
        return this.count

fn expired() -> Weak[Shared[Token]]:
    strong = Shared(Token(7))
    return Weak(strong)

fn main():
    boxed: owns dyn Observe + Bump = owns Holder[bool](true)
    println(boxed.observe(owns expired()))
    println(boxed.bump())
