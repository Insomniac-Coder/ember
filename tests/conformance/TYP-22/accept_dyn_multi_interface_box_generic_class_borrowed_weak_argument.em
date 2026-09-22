#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, DRP-6, HEAP-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ stdout: 1
#$ assert-c: contains("em_vt_dyn_multi__Observe__Bump_Reader_bool_slot0")
#$ assert-c: contains("em_vt_dyn_multi__Observe__Bump_Reader_bool_slot1")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c: !contains("ember_box_new_copy")

# A borrowed Weak[Shared[Token]] argument crossing a composed dynamic box is
# reused by the generic class implementation. The second interface slot must
# still dispatch through the same payload without copying the boxed owner.
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

fn main():
    strong = Shared(Token(7))
    boxed: Box[dyn Observe + Bump] = Box(Reader[bool](true))
    println(boxed.observe(Weak(strong)))
    println(boxed.bump())
