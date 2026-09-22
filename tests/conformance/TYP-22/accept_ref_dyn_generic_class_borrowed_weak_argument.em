#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, BRW-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ assert-c: contains("em_vt_dyn_Observe_Reader_bool_slot0")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# A borrowed Weak[Shared[Token]] argument passed through a ref dyn carrier is
# reused by the generic class implementation. The borrowed class receiver and
# observer remain independent dynamic-interface capabilities.
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

fn inspect(value: ref dyn Observe, weak: Weak[Shared[Token]]) -> i32:
    return value.observe(weak)

fn main():
    strong = Shared(Token(7))
    reader = Reader[bool](true)
    println(inspect(ref reader, Weak(strong)))
