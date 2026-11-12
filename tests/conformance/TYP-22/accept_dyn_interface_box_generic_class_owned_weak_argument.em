#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, DRP-6, OWN-2, HEAP-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, development, release
#$ stdout: 7
#$ assert-c: contains("em_vt_dyn_7Observe_for_6ReaderGda5cc12a188c7619b25d920e0657739a_slot0")
#$ assert-c-count: contains("ember_weak_retain(") == 2
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c: !contains("ember_box_new_copy")

# An owned weakref Token argument crossing a Box[dyn] call receives an
# independent observer in the generic class implementation. The original
# `shares` owner remains separate while the dynamic callee opens and destroys
# its owned argument link.
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

fn main():
    strong = shares mut Token(7)
    weak = weakref strong
    boxed: owns dyn Observe = owns Reader[bool](true)
    println(boxed.observe(owns (weakref weak)))
