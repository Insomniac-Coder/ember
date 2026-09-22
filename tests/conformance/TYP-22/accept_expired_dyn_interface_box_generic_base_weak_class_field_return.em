#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-2, CLS-4, OBJ-3, DRP-6, HEAP-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, WK-14, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ assert-c: contains("em_vt_dyn_Observe_Holder_bool_slot0")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c: !contains("ember_box_new_copy")

# A Weak[Token] returned through a dynamic box from a generic base field stays
# valid after both derived payload and final token owner leave the helper, but
# it must upgrade to None rather than resurrecting the class owner.
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

fn expired() -> Weak[Token]:
    strong = Token(7)
    boxed: Box[dyn Observe] = Box(Holder[bool](Weak(strong), true))
    return boxed.observer()

fn main():
    weak = expired()
    match weak.upgrade():
        Some(_):
            println(0)
        None:
            println(7)
