#$ test: run-pass
#$ rules: TYP-16, CLS-2, CLS-4, OBJ-3, DSP-1, BRW-1, EXC-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, WK-14, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 8
#$ stdout: 5
#$ assert-c: contains("static int32_t em_vt_Derived_bool_slot0")
#$ assert-c: contains("->slot0")
#$ assert-c: contains("ember_object_begin_write")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# Replacing a live inherited weak field through a generic mutable virtual call
# releases only the field's observer. An outside observer to the old token stays
# valid while the override retains and upgrades its replacement.
class Token:
    value: i32

open class Base[T]:
    weak: Weak[Token]
    marker: T
    count: i32

    fn init(mut self, weak: Weak[Token], marker: T, count: i32):
        self.weak = weak
        self.marker = marker
        self.count = count

    virtual fn replace(mut self, replacement: Weak[Token]) -> i32:
        self.weak = replacement
        self.count = self.count + 1
        return self.count

class Derived[T](Base[T]):
    fn init(mut self, weak: Weak[Token], marker: T, count: i32):
        super.init(weak, marker, count)

    override fn replace(mut self, replacement: Weak[Token]) -> i32:
        self.weak = replacement
        self.count = self.count + 1
        match self.weak.upgrade():
            Some(owner):
                return owner.value + self.count
            None:
                return self.count

fn main():
    previous = Token(5)
    replacement = Token(7)
    old = Weak(previous)
    derived = Derived[bool](old, true, 0)
    base: Base[bool] = derived
    println(base.replace(Weak(replacement)))
    match old.upgrade():
        Some(owner):
            println(owner.value)
        None:
            println(0)
