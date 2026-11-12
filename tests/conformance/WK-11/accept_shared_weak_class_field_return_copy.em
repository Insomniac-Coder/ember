#$ test: run-pass
#$ rules: HEAP-3, HEAP-4, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, development, release
#$ stdout: 7
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# Returning a weakref T read from a class field gives the caller its own
# link before the borrowed Holder parameter ends. The returned observer
# may upgrade while the independent strong owner remains live.
struct Token:
    value: i32

class Holder:
    weak: weakref Token

fn observe(holder: Holder) -> weakref Token:
    return weakref holder.weak

fn main():
    strong = shares mut Token(7)
    holder = Holder(weakref strong)
    weak = observe(holder)
    if weak exists as owner:
        println(owner.value)
    else:
        println(0)
