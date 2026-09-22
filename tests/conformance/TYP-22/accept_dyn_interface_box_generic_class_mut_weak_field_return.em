#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, EXC-1, DRP-6, HEAP-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ assert-c: contains("em_vt_dyn_Revise_Observer_bool_slot0")
#$ assert-c: contains("ember_access_begin_write")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c: !contains("ember_box_new_copy")

# A mut interface slot on a generic class updates its private counter and
# returns its Weak[Shared[T]] field through Box[dyn Revise]. The write interval
# closes at the call boundary while the returned observer remains independently
# usable by the caller.
interface Revise:
    fn observer(mut self) -> Weak[Shared[Token]]

struct Token:
    value: i32

class Observer[T] implements Revise:
    weak: Weak[Shared[Token]]
    marker: T
    count: i32

    fn observer(mut self) -> Weak[Shared[Token]]:
        self.count = self.count + 1
        return self.weak

fn main():
    strong = Shared(Token(7))
    boxed: Box[dyn Revise] = Box(Observer[bool](Weak(strong), true, 0))
    weak = boxed.observer()
    match weak.upgrade():
        Some(owner):
            println(owner.get().value)
        None:
            println(0)
