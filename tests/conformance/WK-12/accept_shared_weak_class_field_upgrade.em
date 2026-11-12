#$ test: run-pass
#$ rules: HEAP-3, HEAP-4, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, development, release
#$ stdout: 7
#$ assert-c: contains("ember_weak_upgrade(")

# A weakref T may live in a class field. Reading that field gives a
# borrowed link; upgrading it while the separate strong owner is live
# returns the same `shares` allocation.
struct Token:
    value: i32

class Holder:
    weak: weakref Token

fn main():
    owner = shares mut Token(7)
    holder = Holder(weakref owner)
    if holder.weak exists as live:
        println(live.value)
    else:
        println(0)
