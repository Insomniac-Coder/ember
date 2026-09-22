#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-2, CLS-4, OBJ-3, BRW-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, WK-14, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ assert-c: contains("em_vt_dyn_Observe_Holder_bool_slot0")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# A weak class owner copied out through ref dyn Observe from an inherited base
# field does not preserve the generic derived class or final Token owner. Its
# later upgrade is therefore the valid expired path after helper teardown.
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

fn expired() -> Weak[Token]:
    strong = Token(7)
    holder = Holder[bool](Weak(strong), true)
    return inspect(ref holder)

fn main():
    weak = expired()
    match weak.upgrade():
        Some(_):
            println(0)
        None:
            println(7)
