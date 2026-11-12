#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, BRW-1, OWN-2, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, development, release
#$ stdout: 0
#$ assert-c: contains("em_vt_dyn_7Observe_for_6ReaderGda5cc12a188c7619b25d920e0657739a_slot0")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# An owned weakref Token argument crossing a ref dyn carrier remains
# only a control-block observer after expiry. The generic class receives and
# destroys its independent copy while `exists as` takes None.
interface Observe:
    fn observe(owns weak: weakref Token) -> i32

struct Token:
    value: i32

[T]
class Reader implements Observe:
    marker: T

    fn observe(owns weak: weakref Token) -> i32:
        if weak exists as owner:
            return owner.value
        else:
            return 0

fn expired() -> weakref Token:
    strong = shares mut Token(7)
    return weakref strong

fn inspect(value: ref dyn Observe, weak: weakref Token) -> i32:
    return value.observe(owns (weakref weak))

fn main():
    reader = Reader[bool](true)
    println(inspect(ref reader, expired()))
