#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, DRP-6, HEAP-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, development, release
#$ stdout: 7
#$ assert-c: contains("em_vt_dyn_7Observe_for_8ObserverGda5cc12a188c7619b25d920e0657739a_slot0")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c: !contains("ember_box_new_copy")

# A generic class in Box[dyn Observe] returns its weakref T field through
# an object-safe interface slot. The result is an independent weak observer;
# upgrading it still requires the separate `shares` owner, read as its value.
interface Observe:
    fn observer() -> weakref Token

struct Token:
    value: i32

[T]
class Observer implements Observe:
    weak: weakref Token
    marker: T

    fn observer() -> weakref Token:
        return weakref this.weak

fn main():
    strong = shares mut Token(7)
    boxed: owns dyn Observe = owns Observer[bool](weakref strong, true)
    weak = boxed.observer()
    if weak exists as owner:
        println(owner.value)
    else:
        println(0)
