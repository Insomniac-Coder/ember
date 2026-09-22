#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, CLS-7, DRP-6, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 0
#$ stdout: 1
#$ assert-c: contains("em_vt_dyn_multi__Observe__Bump_Pixel_slot0")
#$ assert-c: contains("em_vt_dyn_multi__Observe__Bump_Pixel_slot1")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c: !contains("ember_box_new_copy")

# A borrowed weak argument through an inherited Pixel dynamic box takes None
# after expiry; the mutable sibling slot still updates the class payload.
interface Observe:
    fn observe(self, weak: Weak[Shared[Token]]) -> i32

interface Bump:
    fn bump(mut self) -> i32

struct Token:
    value: i32

open class Base:
    count: i32

    fn init(mut self):
        self.count = 0

class Pixel(Base) implements Observe, Bump:
    fn init(mut self):
        super.init()

    fn observe(self, weak: Weak[Shared[Token]]) -> i32:
        match weak.upgrade():
            Some(owner):
                return owner.get().value
            None:
                return 0

    fn bump(mut self) -> i32:
        self.count = self.count + 1
        return self.count

fn expired() -> Weak[Shared[Token]]:
    strong = Shared(Token(7))
    return Weak(strong)

fn main():
    boxed: Box[dyn Observe + Bump] = Box(Pixel())
    println(boxed.observe(expired()))
    println(boxed.bump())
