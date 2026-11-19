#$ test: run-pass
#$ rules: HEAP-3, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26, OBJ-3
#$ profiles: debug, development, release
#$ stdout: 7
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# A weakref T returned from a class field remains a valid observer after
# both its source Holder and the last `shares` owner leave scope. It retains only
# the control block, so upgrade must report None and never resurrect Token.
struct Token:
    value: i32

class Holder:
    weak: weakref Token

fn expired() -> weakref Token:
    strong = shares mut Token(7)
    holder = Holder(weakref strong)
    return weakref holder.weak

fn main():
    weak = expired()
    if weak exists as _:
        println(0)
    else:
        println(7)
