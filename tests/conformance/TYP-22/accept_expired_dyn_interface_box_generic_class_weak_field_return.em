#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, DRP-6, HEAP-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, development, release
#$ stdout: 7
#$ assert-c: contains("em_vt_dyn_7Observe_for_8ObserverGda5cc12a188c7619b25d920e0657739a_slot0")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c: !contains("ember_box_new_copy")

# A weak observer returned from a Box[dyn Observe] adapter remains valid after
# both the box payload and its last `shares` owner leave scope. It retains only
# the control block, so the later upgrade must take the None arm.
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

fn expired() -> weakref Token:
    strong = shares mut Token(7)
    boxed: owns dyn Observe = owns Observer[bool](weakref strong, true)
    return boxed.observer()

fn main():
    weak = expired()
    if weak exists as _:
        println(0)
    else:
        println(7)
