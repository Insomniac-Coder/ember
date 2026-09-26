#$ test: run-pass
#$ rules: TYP-16, CLS-2, CLS-4, DSP-1, BRW-1, EXC-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 8
#$ assert-c: contains("static int32_t em_vt_Derived_bool_slot0")
#$ assert-c: contains("->slot0")
#$ assert-c: contains("ember_object_begin_write")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# A base-typed generic handle dispatches a mut virtual call that replaces its
# inherited Weak[Shared[T]] field. The override owns the new observer, drops
# the previous empty observer, and upgrades only the replacement.
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

    virtual fn replace(mut self, replacement: Weak[Shared[Token]]) -> i32:
        self.weak = replacement
        self.count = self.count + 1
        return self.count

class Derived[T](Base[T]):
    fn init(mut self, weak: Weak[Shared[Token]], marker: T, count: i32):
        super.init(weak, marker, count)

    override fn replace(mut self, replacement: Weak[Shared[Token]]) -> i32:
        self.weak = replacement
        self.count = self.count + 1
        match self.weak.upgrade():
            Some(owner):
                return owner.get().value + self.count
            None:
                return self.count

fn main():
    strong = Shared(Token(7))
    empty: Weak[Shared[Token]] = Weak[Shared[Token]].empty()
    derived = Derived[bool](empty, true, 0)
    base: Base[bool] = derived
    println(base.replace(Weak(strong)))
