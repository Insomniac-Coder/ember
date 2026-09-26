#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-2, CLS-4, OBJ-3, DRP-6, EXC-1, HEAP-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, WK-14, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 1
#$ stdout: 7
#$ assert-c: contains("em_vt_dyn_multi__Observe__Bump_Holder_bool_slot0")
#$ assert-c: contains("em_vt_dyn_multi__Observe__Bump_Holder_bool_slot1")
#$ assert-c: contains("ember_object_begin_write")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c: !contains("ember_box_new_copy")

# A multi-interface dynamic box may return an inherited Weak[Token] field from
# its generic base and then mutate a separate inherited counter. Both ordered
# slots share the one derived payload without copying the box or class owner.
interface Observe:
    fn observer(self) -> Weak[Token]

interface Bump:
    fn bump(mut self) -> i32

class Token:
    value: i32

open class Base[T]:
    weak: Weak[Token]
    marker: T
    count: i32

    fn init(mut self, weak: Weak[Token], marker: T):
        self.weak = weak
        self.marker = marker
        self.count = 0

class Holder[T](Base[T]) implements Observe, Bump:
    fn init(mut self, weak: Weak[Token], marker: T):
        super.init(weak, marker)

    fn observer(self) -> Weak[Token]:
        return self.weak

    fn bump(mut self) -> i32:
        self.count = self.count + 1
        return self.count

fn main():
    strong = Token(7)
    boxed: Box[dyn Observe + Bump] = Box(Holder[bool](Weak(strong), true))
    weak = boxed.observer()
    println(boxed.bump())
    match weak.upgrade():
        Some(owner):
            println(owner.value)
        None:
            println(0)
