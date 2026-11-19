#$ test: run-pass
#$ rules: OBJ-3, HEAP-7, WK-11, WK-12, WK-14, TST-26
#$ profiles: debug, development, release
#$ stdout: 7
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# A weakref C returned from a class field remains a valid but expired observer
# once both the source Holder and its final Token owner leave the helper. Its
# control block survives solely for the returned link; upgrade is None.
class Token:
    value: i32

class Holder:
    weak: weakref Token

fn expired() -> weakref Token:
    strong = Token(7)
    holder = Holder(weakref strong)
    return weakref holder.weak

fn main():
    weak = expired()
    if weak exists as _:
        println(0)
    else:
        println(7)
