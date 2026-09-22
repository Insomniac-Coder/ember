#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, DRP-6, HEAP-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ assert-c: contains("em_vt_dyn_Observe_Observer_bool_slot0")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c: !contains("ember_box_new_copy")

# A generic class in Box[dyn Observe] returns its Weak[Shared[T]] field through
# an object-safe interface slot. The result is an independent weak observer;
# upgrading it still requires the separate Shared owner and explicit get().
interface Observe:
    fn observer(self) -> Weak[Shared[Token]]

struct Token:
    value: i32

class Observer[T] implements Observe:
    weak: Weak[Shared[Token]]
    marker: T

    fn observer(self) -> Weak[Shared[Token]]:
        return self.weak

fn main():
    strong = Shared(Token(7))
    boxed: Box[dyn Observe] = Box(Observer[bool](Weak(strong), true))
    weak = boxed.observer()
    match weak.upgrade():
        Some(owner):
            println(owner.get().value)
        None:
            println(0)
