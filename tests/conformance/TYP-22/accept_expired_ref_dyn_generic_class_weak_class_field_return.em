#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, OBJ-3, BRW-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, WK-14, TST-26
#$ profiles: debug, development, release
#$ stdout: 7
#$ assert-c: contains("em_vt_dyn_7Observe_for_6HolderGda5cc12a188c7619b25d920e0657739a_slot0")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# A weakref Token copied out through ref dyn Observe remains non-owning after the
# borrowed generic class and its final Token owner leave the helper. The caller
# observes the expired upgrade path without any class-owner resurrection.
interface Observe:
    fn observer() -> weakref Token

class Token:
    value: i32

[T]
class Holder implements Observe:
    weak: weakref Token
    marker: T

    fn observer() -> weakref Token:
        return weakref this.weak

fn inspect(ref value: dyn Observe) -> weakref Token:
    return value.observer()

fn expired() -> weakref Token:
    strong = Token(7)
    holder = Holder[bool](weakref strong, true)
    return inspect(ref holder)

fn main():
    weak = expired()
    if weak exists as _:
        println(0)
    else:
        println(7)
