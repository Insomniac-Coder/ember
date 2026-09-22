#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-2, CLS-4, DRP-6, OWN-2, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 0
#$ stdout: 1
#$ assert-c: contains("em_vt_dyn_multi__Observe__Bump_Holder_bool_slot0")
#$ assert-c: contains("em_vt_dyn_multi__Observe__Bump_Holder_bool_slot1")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c: !contains("ember_box_new_copy")

# An owned weak argument through a composed dynamic box with generic-base state
# receives and destroys an independent observer after expiry. The inherited
# counter remains callable through the second interface slot.
interface Observe:
    fn observe(self, owned weak: Weak[Shared[Token]]) -> i32

interface Bump:
    fn bump(mut self) -> i32

struct Token:
    value: i32

open class Base[T]:
    marker: T
    count: i32

    fn init(mut self, marker: T):
        self.marker = marker
        self.count = 0

class Holder[T](Base[T]) implements Observe, Bump:
    fn init(mut self, marker: T):
        super.init(marker)

    fn observe(self, owned weak: Weak[Shared[Token]]) -> i32:
        match weak.upgrade():
            Some(owner):
                return owner.get().value
            None:
                return 0

    fn bump(mut self) -> i32:
        self.count = self.count + 1
        return self.count

fn expired() -> Weak[Shared[Token]]:
    strong = Shared(Token(7))
    return Weak(strong)

fn main():
    boxed: Box[dyn Observe + Bump] = Box(Holder[bool](true))
    println(boxed.observe(expired()))
    println(boxed.bump())
