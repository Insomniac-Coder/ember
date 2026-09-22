#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, DRP-6, HEAP-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, WK-14, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ assert-c: contains("em_vt_dyn_Observe_Holder_bool_slot0")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c: !contains("ember_box_new_copy")

# A Weak[Token] returned by Box[dyn Observe] outlives the generic class payload
# and its final class owner without keeping either alive. The later upgrade must
# take the expired path rather than reconstructing a class handle.
interface Observe:
    fn observer(self) -> Weak[Token]

class Token:
    value: i32

class Holder[T] implements Observe:
    weak: Weak[Token]
    marker: T

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
