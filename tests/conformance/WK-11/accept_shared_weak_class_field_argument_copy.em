#$ test: run-pass
#$ rules: HEAP-3, HEAP-4, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, development, release
#$ stdout: 7
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# Passing a weakref T class field to an `owned` parameter copies its weak
# handle for the callee. The copied observer may upgrade while the caller's
# separate `shares` owner is live, but it never turns the field read into a
# payload copy.
struct Token:
    value: i32

class Holder:
    weak: weakref Token

fn observe(owns weak: weakref Token) -> i32:
    if weak exists as owner:
        return owner.value
    else:
        return 0

fn main():
    strong = shares mut Token(7)
    holder = Holder(weakref strong)
    println(observe(owns (weakref holder.weak)))
