#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, OBJ-3, BRW-1, EXC-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, WK-14, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ assert-c: contains("em_vt_dyn_Revise_Holder_bool_slot0")
#$ assert-c: contains("ember_object_begin_write")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# A ref mut dyn Revise carrier updates a generic class and returns its
# Weak[Token] field. The receiver remains borrowed and the copied weak observer
# may upgrade only while the separate Token owner stays alive.
interface Revise:
    fn observer(mut self) -> Weak[Token]

class Token:
    value: i32

class Holder[T] implements Revise:
    weak: Weak[Token]
    marker: T
    count: i32

    fn observer(mut self) -> Weak[Token]:
        self.count = self.count + 1
        return self.weak

fn inspect(value: ref mut dyn Revise) -> Weak[Token]:
    return value.observer()

fn main():
    strong = Token(7)
    holder = Holder[bool](Weak(strong), true, 0)
    weak = inspect(ref mut holder)
    match weak.upgrade():
        Some(owner):
            println(owner.value)
        None:
            println(0)
