#$ test: run-pass
#$ rules: TYP-16, CLS-2, CLS-4, DSP-1, BRW-1, EXC-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 9
#$ assert-c: contains("static int32_t em_vt_Derived_bool_slot0")
#$ assert-c: contains("->slot0")
#$ assert-c: contains("ember_access_begin_write")
#$ assert-c: contains("ember_weak_upgrade(")

# A base-typed generic handle dispatches a mut virtual call to the derived
# implementation. The override updates the inherited count and observes the
# inherited Weak[Shared[T]] field through the explicit Shared.get() boundary.
struct Token:
    value: i32

open class Base[T]:
    weak: Weak[Shared[Token]]
    marker: T
    count: i32

    fn init(mut self, weak: Weak[Shared[Token]], marker: T, count: i32):
        self.weak = weak
        self.marker = marker
        self.count = count

    virtual fn tick(mut self) -> i32:
        self.count = self.count + 1
        return self.count

class Derived[T](Base[T]):
    fn init(mut self, weak: Weak[Shared[Token]], marker: T, count: i32):
        super.init(weak, marker, count)

    override fn tick(mut self) -> i32:
        self.count = self.count + 2
        match self.weak.upgrade():
            Some(owner):
                return owner.get().value + self.count
            None:
                return self.count

fn main():
    strong = Shared(Token(7))
    derived = Derived[bool](Weak(strong), true, 0)
    base: Base[bool] = derived
    println(base.tick())
