#$ test: run-pass
#$ rules: TYP-16, CLS-2, CLS-4, OBJ-3, DSP-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, WK-14, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ assert-c: contains("static em_Weak_Token em_vt_Derived_bool_slot0")
#$ assert-c: contains("->slot0")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# A generic base-typed virtual call returns an inherited Weak[Token] field from
# its derived override. The returned observer is independent of the virtual
# receiver and upgrades only while the separate class owner remains live.
class Token:
    value: i32

open class Base[T]:
    weak: Weak[Token]
    marker: T

    fn init(mut self, weak: Weak[Token], marker: T):
        self.weak = weak
        self.marker = marker

    virtual fn observer(self) -> Weak[Token]:
        return self.weak

class Derived[T](Base[T]):
    fn init(mut self, weak: Weak[Token], marker: T):
        super.init(weak, marker)

    override fn observer(self) -> Weak[Token]:
        return self.weak

fn main():
    strong = Token(7)
    derived = Derived[bool](Weak(strong), true)
    base: Base[bool] = derived
    weak = base.observer()
    match weak.upgrade():
        Some(owner):
            println(owner.value)
        None:
            println(0)
