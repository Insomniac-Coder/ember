#$ test: run-pass
#$ rules: TYP-16, CLS-2, CLS-4, DSP-1, OWN-2, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, WK-14, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 0
#$ assert-c: contains("static int32_t em_vt_Derived_bool_slot0")
#$ assert-c: contains("->slot0")
#$ assert-c-count: contains("ember_weak_retain(") == 2
#$ assert-c: contains("ember_weak_release((ember_obj_header*)_2.value);")
#$ assert-c: contains("ember_weak_upgrade(")

# An owned weak argument crossing a generic virtual call remains only a
# control-block observer after its Shared target has expired. The derived
# override must take the None path and release its independent argument copy.
struct Token:
    value: i32

class Holder:
    weak: Weak[Shared[Token]]

open class Base[T]:
    marker: T

    fn init(mut self, marker: T):
        self.marker = marker

    virtual fn observe(self, owned weak: Weak[Shared[Token]]) -> i32:
        match weak.upgrade():
            Some(owner):
                return owner.get().value
            None:
                return 0

class Derived[T](Base[T]):
    fn init(mut self, marker: T):
        super.init(marker)

    override fn observe(self, owned weak: Weak[Shared[Token]]) -> i32:
        match weak.upgrade():
            Some(owner):
                return owner.get().value
            None:
                return 0

fn expired_holder() -> Holder:
    strong = Shared(Token(7))
    return Holder(Weak(strong))

fn main():
    holder = expired_holder()
    derived = Derived[bool](true)
    base: Base[bool] = derived
    println(base.observe(holder.weak))
