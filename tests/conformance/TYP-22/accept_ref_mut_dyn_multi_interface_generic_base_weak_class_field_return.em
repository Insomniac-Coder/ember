#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-2, CLS-4, OBJ-3, BRW-1, EXC-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, WK-14, TST-26
#$ profiles: debug, development, release
#$ stdout: 7
#$ assert-c: contains("em_vt_dyn_7Observe_4Bump_for_6HolderGda5cc12a188c7619_slot0")
#$ assert-c: contains("em_vt_dyn_7Observe_4Bump_for_6HolderGda5cc12a188c7619_slot1")
#$ assert-c: contains("ember_object_begin_write")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# A ref mut dyn Observe + Bump carrier mutates an inherited generic-base
# counter, then returns its inherited Weak[Token] field through the distinct
# shared slot. The borrowed class is never copied; only the weak observer is.
interface Observe:
    fn observer() -> Weak[Token]

interface Bump:
    fn bump() -> i32

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
class Holder(Base[T]) implements Observe, Bump:
    fn init(weak: Weak[Token], marker: T):
        super.init(weak, marker)

    fn observer() -> Weak[Token]:
        return this.weak

    fn bump() -> i32:
        this.count = this.count + 1
        return this.count

fn inspect(ref mut value: dyn Observe + Bump) -> Weak[Token]:
    value.bump()
    return value.observer()

fn main():
    strong = Token(7)
    holder = Holder[bool](Weak(strong), true)
    weak = inspect(ref mut holder)
    match weak.upgrade():
        Some(owner):
            println(owner.value)
        None:
            println(0)
