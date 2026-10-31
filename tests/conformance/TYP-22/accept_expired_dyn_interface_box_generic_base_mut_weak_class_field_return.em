#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-2, CLS-4, OBJ-3, DRP-6, EXC-1, HEAP-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, WK-14, TST-26
#$ profiles: debug, development, release
#$ stdout: 7
#$ assert-c: contains("em_vt_dyn_6Revise_for_6HolderGda5cc12a188c7619_slot0")
#$ assert-c: contains("ember_object_begin_write")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c: !contains("ember_box_new_copy")

# A mutable dynamic call may return Weak[Token] from a generic base field after
# updating an inherited counter. When the box and final Token owner leave the
# helper, the returned observer must follow the expired upgrade path.
interface Revise:
    fn observer() -> Weak[Token]

class Token:
    value: i32

[T]
open class Base:
    weak: Weak[Token]
    marker: T
    count: i32

    fn init(weak: Weak[Token], marker: T):
        this.weak = weak
        this.marker = marker
        this.count = 0

[T]
class Holder(Base[T]) implements Revise:
    fn init(weak: Weak[Token], marker: T):
        super.init(weak, marker)

    fn observer() -> Weak[Token]:
        this.count = this.count + 1
        return this.weak

fn expired() -> Weak[Token]:
    strong = Token(7)
    boxed: Box[dyn Revise] = Box(Holder[bool](Weak(strong), true))
    return boxed.observer()

fn main():
    weak = expired()
    match weak.upgrade():
        Some(_):
            println(0)
        None:
            println(7)
