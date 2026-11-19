#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, BRW-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, development, release
#$ stdout: 7
#$ assert-c: contains("em_vt_dyn_7Observe_for_8ObserverGda5cc12a188c7619b25d920e0657739a_slot0")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# A ref dyn Observe call returns a weak field from a generic class without
# extending either the class or payload lifetime. Once the helper exits, the
# returned observer can only take the expired upgrade path.
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

fn inspect(ref value: dyn Observe) -> weakref Token:
    return value.observer()

fn expired() -> weakref Token:
    strong = shares mut Token(7)
    observer = Observer[bool](weakref strong, true)
    return inspect(ref observer)

fn main():
    weak = expired()
    if weak exists as owner:
        println(owner.value)
    else:
        println(7)
