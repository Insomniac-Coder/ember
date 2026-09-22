#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, DRP-6, HEAP-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ assert-c: contains("em_vt_dyn_Observe_Observer_bool_slot0")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c: !contains("ember_box_new_copy")

# A weak observer returned from a Box[dyn Observe] adapter remains valid after
# both the box payload and its last Shared owner leave scope. It retains only
# the control block, so the later upgrade must take the None arm.
interface Observe:
    fn observer(self) -> Weak[Shared[Token]]

struct Token:
    value: i32

class Observer[T] implements Observe:
    weak: Weak[Shared[Token]]
    marker: T

    fn observer(self) -> Weak[Shared[Token]]:
        return self.weak

fn expired() -> Weak[Shared[Token]]:
    strong = Shared(Token(7))
    boxed: Box[dyn Observe] = Box(Observer[bool](Weak(strong), true))
    return boxed.observer()

fn main():
    weak = expired()
    match weak.upgrade():
        Some(_):
            println(0)
        None:
            println(7)
