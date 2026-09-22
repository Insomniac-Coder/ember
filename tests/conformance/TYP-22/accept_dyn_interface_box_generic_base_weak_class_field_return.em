#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-2, CLS-4, OBJ-3, DRP-6, HEAP-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, WK-14, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ assert-c: contains("em_vt_dyn_Observe_Holder_bool_slot0")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c: !contains("ember_box_new_copy")

# A dynamic box may return a Weak[Token] field inherited from a generic base.
# The adapter sees one derived payload while base initialization and teardown
# retain and release the class-owner observer normally.
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

fn main():
    strong = Token(7)
    boxed: Box[dyn Observe] = Box(Holder[bool](Weak(strong), true))
    weak = boxed.observer()
    match weak.upgrade():
        Some(owner):
            println(owner.value)
        None:
            println(0)
