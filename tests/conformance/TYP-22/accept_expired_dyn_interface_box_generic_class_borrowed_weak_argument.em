#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, DRP-6, HEAP-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 0
#$ assert-c: contains("em_vt_dyn_Observe_Reader_bool_slot0")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c: !contains("ember_box_new_copy")

# A borrowed Weak[Shared[Token]] argument crossing a Box[dyn] call remains a
# valid non-owning observer after its Shared target expires. The generic class
# implementation must take the None path without copying at dispatch.
interface Observe:
    fn observe(self, weak: Weak[Shared[Token]]) -> i32

struct Token:
    value: i32

class Reader[T] implements Observe:
    marker: T

    fn observe(self, weak: Weak[Shared[Token]]) -> i32:
        match weak.upgrade():
            Some(owner):
                return owner.get().value
            None:
                return 0

fn expired() -> Weak[Shared[Token]]:
    strong = Shared(Token(7))
    return Weak(strong)

fn main():
    boxed: Box[dyn Observe] = Box(Reader[bool](true))
    println(boxed.observe(expired()))
