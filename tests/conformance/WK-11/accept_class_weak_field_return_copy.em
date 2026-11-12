#$ test: run-pass
#$ rules: OBJ-3, HEAP-7, WK-11, WK-12, WK-14, TST-26
#$ profiles: debug, development, release
#$ stdout: 7
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# Returning weakref C from a borrowed class field gives the caller its own weak
# handle before the source Holder parameter ends. That copy remains non-owning
# and may upgrade only while a separate class owner is live.
class Token:
    value: i32

class Holder:
    weak: weakref Token

fn observe(holder: Holder) -> weakref Token:
    return weakref holder.weak

fn main():
    strong = Token(7)
    holder = Holder(weakref strong)
    weak = observe(holder)
    if weak exists as owner:
        println(owner.value)
    else:
        println(0)
