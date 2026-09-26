#$ test: run-pass
#$ rules: TYP-16, CLS-2, CLS-4, OBJ-3, DSP-1, BRW-1, EXC-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, WK-14, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 2
#$ assert-c: contains("static int32_t em_vt_Derived_bool_slot0")
#$ assert-c: contains("->slot0")
#$ assert-c: contains("ember_object_begin_write")
#$ assert-c: contains("ember_weak_upgrade(")

# A base-typed generic handle dispatches a mut virtual call after its inherited
# Weak[Token] field has expired. The derived override still updates its counter,
# but the weak upgrade takes None without reconstructing the former class owner.
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

    virtual fn tick(mut self) -> i32:
        self.count = self.count + 1
        return self.count

class Derived[T](Base[T]):
    fn init(mut self, weak: Weak[Token], marker: T, count: i32):
        super.init(weak, marker, count)

    override fn tick(mut self) -> i32:
        self.count = self.count + 2
        match self.weak.upgrade():
            Some(owner):
                return owner.value + self.count
            None:
                return self.count

fn expired() -> Weak[Token]:
    strong = Token(7)
    return Weak(strong)

fn main():
    derived = Derived[bool](expired(), true, 0)
    base: Base[bool] = derived
    println(base.tick())
