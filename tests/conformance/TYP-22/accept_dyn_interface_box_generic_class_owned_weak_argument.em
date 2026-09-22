#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, DRP-6, OWN-2, HEAP-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ assert-c: contains("em_vt_dyn_Observe_Reader_bool_slot0")
#$ assert-c-count: contains("ember_weak_retain(") == 2
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c: !contains("ember_box_new_copy")

# An owned Weak[Shared[Token]] argument crossing a Box[dyn] call receives an
# independent observer in the generic class implementation. The original
# Shared owner remains separate while the dynamic callee upgrades and destroys
# its owned argument copy.
interface Observe:
    fn observe(self, owned weak: Weak[Shared[Token]]) -> i32

struct Token:
    value: i32

class Reader[T] implements Observe:
    marker: T

    fn observe(self, owned weak: Weak[Shared[Token]]) -> i32:
        match weak.upgrade():
            Some(owner):
                return owner.get().value
            None:
                return 0

fn main():
    strong = Shared(Token(7))
    weak = Weak(strong)
    boxed: Box[dyn Observe] = Box(Reader[bool](true))
    println(boxed.observe(weak))
