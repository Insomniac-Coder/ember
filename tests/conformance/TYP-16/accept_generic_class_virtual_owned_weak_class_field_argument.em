#$ test: run-pass
#$ rules: TYP-16, CLS-2, CLS-4, OBJ-3, DSP-1, OWN-2, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, WK-14, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ assert-c: contains("static int32_t em_vt_Derived_bool_slot0")
#$ assert-c: contains("->slot0")
#$ assert-c-count: contains("ember_weak_retain(") == 2
#$ assert-c: contains("ember_weak_release((ember_obj_header*)_2.value);")
#$ assert-c: contains("ember_weak_upgrade(")

# Passing Weak[Token] to an owned generic virtual parameter copies one weak
# observer for the derived override. The Holder keeps its field observer while
# the callee independently upgrades and then destroys its owned weak argument.
class Token:
    value: i32

class Holder:
    weak: Weak[Token]

open class Base[T]:
    marker: T

    fn init(mut self, marker: T):
        self.marker = marker

    virtual fn observe(self, owned weak: Weak[Token]) -> i32:
        match weak.upgrade():
            Some(owner):
                return owner.value
            None:
                return 0

class Derived[T](Base[T]):
    fn init(mut self, marker: T):
        super.init(marker)

    override fn observe(self, owned weak: Weak[Token]) -> i32:
        match weak.upgrade():
            Some(owner):
                return owner.value
            None:
                return 0

fn main():
    strong = Token(7)
    holder = Holder(Weak(strong))
    derived = Derived[bool](true)
    base: Base[bool] = derived
    println(base.observe(holder.weak))
