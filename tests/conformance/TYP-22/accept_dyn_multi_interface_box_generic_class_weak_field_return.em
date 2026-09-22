#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, DRP-6, EXC-1, HEAP-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 1
#$ stdout: 7
#$ assert-c: contains("em_vt_dyn_multi__Observe__Bump_Counter_bool_slot0")
#$ assert-c: contains("em_vt_dyn_multi__Observe__Bump_Counter_bool_slot1")
#$ assert-c: contains("ember_access_begin_write")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c: !contains("ember_box_new_copy")

# One Box[dyn Observe + Bump] carrier can return a generic class's weak field
# through its shared slot, then call its separate mutable slot. The returned
# observer remains independent while the write interval is confined to bump().
interface Observe:
    fn observer(self) -> Weak[Shared[Token]]

interface Bump:
    fn bump(mut self) -> i32

struct Token:
    value: i32

class Counter[T] implements Observe, Bump:
    weak: Weak[Shared[Token]]
    marker: T
    count: i32

    fn observer(self) -> Weak[Shared[Token]]:
        return self.weak

    fn bump(mut self) -> i32:
        self.count = self.count + 1
        return self.count

fn main():
    strong = Shared(Token(7))
    boxed: Box[dyn Observe + Bump] = Box(Counter[bool](Weak(strong), true, 0))
    weak = boxed.observer()
    println(boxed.bump())
    match weak.upgrade():
        Some(owner):
            println(owner.get().value)
        None:
            println(0)
