#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, EXC-1, DRP-6, HEAP-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, development, release
#$ stdout: 7
#$ assert-c: contains("em_vt_dyn_6Revise_for_8ObserverGda5cc12a188c7619b25d920e0657739a_slot0")
#$ assert-c: contains("ember_object_begin_write")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c: !contains("ember_box_new_copy")

# A mut interface slot on a generic class updates its private counter and
# returns its weakref T field through Box[dyn Revise]. The write interval
# closes at the call boundary while the returned observer remains independently
# usable by the caller.
interface Revise:
    fn observer() -> weakref Token

struct Token:
    value: i32

[T]
class Observer implements Revise:
    weak: weakref Token
    marker: T
    count: i32

    fn observer() -> weakref Token:
        this.count = this.count + 1
        return weakref this.weak

fn main():
    strong = shares mut Token(7)
    boxed: owns dyn Revise = owns Observer[bool](weakref strong, true, 0)
    weak = boxed.observer()
    if weak exists as owner:
        println(owner.value)
    else:
        println(0)
