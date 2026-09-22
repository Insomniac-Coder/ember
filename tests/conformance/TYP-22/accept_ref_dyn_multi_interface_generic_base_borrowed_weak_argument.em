#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-2, CLS-4, DRP-6, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 8
#$ assert-c: contains("em_vt_dyn_multi__Observe__Bump_Holder_bool_slot0")
#$ assert-c: contains("em_vt_dyn_multi__Observe__Bump_Holder_bool_slot1")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# A generic derived class with inherited state can be borrowed as ref dyn
# Observe + Bump while receiving a borrowed weak argument. Both ordered slots
# dispatch through the one payload and the inherited counter is updated.
interface Observe:
    fn observe(self, weak: Weak[Shared[Token]]) -> i32

interface Bump:
    fn bump(self) -> i32

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

    fn observe(self, weak: Weak[Shared[Token]]) -> i32:
        match weak.upgrade():
            Some(owner):
                return owner.get().value
            None:
                return 0

    fn bump(self) -> i32:
        return 1

fn inspect(value: ref dyn Observe + Bump, weak: Weak[Shared[Token]]) -> i32:
    return value.observe(weak) + value.bump()

fn main():
    strong = Shared(Token(7))
    holder = Holder[bool](true)
    println(inspect(ref holder, Weak(strong)))
