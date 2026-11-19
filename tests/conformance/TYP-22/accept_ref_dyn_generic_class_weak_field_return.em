#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, BRW-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, development, release
#$ stdout: 7
#$ assert-c: contains("em_vt_dyn_7Observe_for_8ObserverGda5cc12a188c7619b25d920e0657739a_slot0")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# A ref dyn Observe call returns an independent weakref T copy from a
# generic class field. The erased carrier borrows the class handle, while the
# returned weak observer remains valid after the ref dyn call ends.
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

fn inspect(value: ref dyn Observe) -> weakref Token:
    return value.observer()

fn main():
    strong = shares mut Token(7)
    observer = Observer[bool](weakref strong, true)
    weak = inspect(ref observer)
    if weak exists as owner:
        println(owner.value)
    else:
        println(0)
