#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-2, CLS-4, OBJ-3, BRW-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, WK-14, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ assert-c: contains("em_vt_dyn_Observe_Holder_bool_slot0")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# A ref dyn Observe carrier borrows a generic derived class and returns its
# inherited Weak[Token] base field. Only the weak observer gains independent
# lifetime; neither the borrowed derived handle nor the Token owner is copied.
interface Observe:
    fn observer(self) -> Weak[Token]

class Token:
    value: i32

open class Base[T]:
    weak: Weak[Token]
    marker: T

    fn init(mut self, weak: Weak[Token], marker: T):
        self.weak = weak
        self.marker = marker

class Holder[T](Base[T]) implements Observe:
    fn init(mut self, weak: Weak[Token], marker: T):
        super.init(weak, marker)

    fn observer(self) -> Weak[Token]:
        return self.weak

fn inspect(value: ref dyn Observe) -> Weak[Token]:
    return value.observer()

fn main():
    strong = Token(7)
    holder = Holder[bool](Weak(strong), true)
    weak = inspect(ref holder)
    match weak.upgrade():
        Some(owner):
            println(owner.value)
        None:
            println(0)
