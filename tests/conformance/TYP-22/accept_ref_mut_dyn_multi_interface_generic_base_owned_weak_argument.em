#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-2, CLS-4, BRW-1, EXC-1, OWN-2, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 8
#$ assert-c: contains("em_vt_dyn_multi__Observe__Bump_Holder_bool_slot0")
#$ assert-c: contains("em_vt_dyn_multi__Observe__Bump_Holder_bool_slot1")
#$ assert-c: contains("ember_access_begin_write")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# A generic derived class with inherited state crosses a ref mut composed
# carrier with an owned weak argument. The mutable Observe slot updates the
# inherited counter and the Bump slot observes that same payload.
interface Observe:
    fn observe(mut self, owned weak: Weak[Shared[Token]]) -> i32

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

    fn observe(mut self, owned weak: Weak[Shared[Token]]) -> i32:
        self.count = self.count + 1
        match weak.upgrade():
            Some(owner):
                return owner.get().value
            None:
                return 0

    fn bump(mut self) -> i32:
        return self.count

fn inspect(value: ref mut dyn Observe + Bump, weak: Weak[Shared[Token]]) -> i32:
    return value.observe(weak) + value.bump()

fn main():
    strong = Shared(Token(7))
    holder = Holder[bool](true)
    println(inspect(ref mut holder, Weak(strong)))
