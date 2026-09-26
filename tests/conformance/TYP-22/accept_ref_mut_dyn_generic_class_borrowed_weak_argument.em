#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, BRW-1, EXC-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ assert-c: contains("em_vt_dyn_Observe_Reader_bool_slot0")
#$ assert-c: contains("ember_object_begin_write")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# A borrowed Weak[Shared[Token]] argument crossing a ref mut dyn carrier is
# reused by the generic class implementation while the receiver's write
# interval remains scoped to the dynamic call.
interface Observe:
    fn observe(mut self, weak: Weak[Shared[Token]]) -> i32

struct Token:
    value: i32

class Reader[T] implements Observe:
    marker: T
    count: i32

    fn observe(mut self, weak: Weak[Shared[Token]]) -> i32:
        self.count = self.count + 1
        match weak.upgrade():
            Some(owner):
                return owner.get().value
            None:
                return 0

fn inspect(value: ref mut dyn Observe, weak: Weak[Shared[Token]]) -> i32:
    return value.observe(weak)

fn main():
    strong = Shared(Token(7))
    reader = Reader[bool](true, 0)
    println(inspect(ref mut reader, Weak(strong)))
