#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, DRP-6, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 0
#$ stdout: 1
#$ assert-c: contains("em_vt_dyn_multi__Observe__Bump_Reader_bool_slot0")
#$ assert-c: contains("em_vt_dyn_multi__Observe__Bump_Reader_bool_slot1")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c: !contains("ember_box_new_copy")

# A borrowed Weak[Shared[Token]] argument through a composed dynamic box stays
# a valid observer after its target expires. The generic implementation must
# take the None path while the second interface slot remains callable.
interface Observe:
    fn observe(self, weak: Weak[Shared[Token]]) -> i32

interface Bump:
    fn bump(self) -> i32

struct Token:
    value: i32

class Reader[T] implements Observe, Bump:
    marker: T

    fn observe(self, weak: Weak[Shared[Token]]) -> i32:
        match weak.upgrade():
            Some(owner):
                return owner.get().value
            None:
                return 0

    fn bump(self) -> i32:
        return 1

fn expired() -> Weak[Shared[Token]]:
    strong = Shared(Token(7))
    return Weak(strong)

fn main():
    boxed: Box[dyn Observe + Bump] = Box(Reader[bool](true))
    println(boxed.observe(expired()))
    println(boxed.bump())
