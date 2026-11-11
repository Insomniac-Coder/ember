#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-2, CLS-4, OBJ-3, DRP-6, HEAP-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, WK-14, TST-26
#$ profiles: debug, development, release
#$ stdout: 7
#$ assert-c: contains("em_vt_dyn_7Observe_for_6HolderGda5cc12a188c7619b25d920e0657739a_slot0")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c: !contains("ember_box_new_copy")

# A Weak[Token] returned through a dynamic box from a generic base field stays
# valid after both derived payload and final token owner leave the helper, but
# it must upgrade to None rather than resurrecting the class owner.
interface Observe:
    fn observer() -> Weak[Token]

class Token:
    value: i32

[T]
open class Base:
    weak: Weak[Token]
    marker: T

    fn init(weak: Weak[Token], marker: T):
        this.weak = weak
        this.marker = marker

[T]
class Holder(Base[T]) implements Observe:
    fn init(weak: Weak[Token], marker: T):
        super.init(weak, marker)

    fn observer() -> Weak[Token]:
        return this.weak

fn expired() -> Weak[Token]:
    strong = Token(7)
    boxed: owns dyn Observe = owns Holder[bool](Weak(strong), true)
    return boxed.observer()

fn main():
    weak = expired()
    match weak.upgrade():
        Some(_):
            println(0)
        None:
            println(7)
