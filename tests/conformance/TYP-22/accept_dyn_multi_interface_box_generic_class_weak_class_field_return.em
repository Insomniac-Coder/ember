#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, OBJ-3, DRP-6, EXC-1, HEAP-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, WK-14, TST-26
#$ profiles: debug, development, release
#$ stdout: 1
#$ stdout: 7
#$ assert-c: contains("em_vt_dyn_7Observe_4Bump_for_6HolderGda5cc12a188c7619b25d920e0657739a_slot0")
#$ assert-c: contains("em_vt_dyn_7Observe_4Bump_for_6HolderGda5cc12a188c7619b25d920e0657739a_slot1")
#$ assert-c: contains("ember_object_begin_write")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c: !contains("ember_box_new_copy")

# One Box[dyn Observe + Bump] can return weakref Token through its shared slot,
# then invoke its mutable slot on the same generic class payload. The observer
# remains separate from the box and upgrades only against the live Token owner.
interface Observe:
    fn observer() -> weakref Token

interface Bump:
    fn bump() -> i32

class Token:
    value: i32

[T]
class Holder implements Observe, Bump:
    weak: weakref Token
    marker: T
    count: i32

    fn observer() -> weakref Token:
        return weakref this.weak

    fn bump() -> i32:
        this.count = this.count + 1
        return this.count

fn main():
    strong = Token(7)
    boxed: owns dyn Observe + Bump = owns Holder[bool](weakref strong, true, 0)
    weak = boxed.observer()
    println(boxed.bump())
    if weak exists as owner:
        println(owner.value)
    else:
        println(0)
