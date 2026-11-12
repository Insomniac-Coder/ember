#$ test: run-pass
#$ rules: FN-1, HEAP-3, HEAP-4, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, development, release
#$ stdout: 7
#$ assert-c-count: contains("ember_weak_retain(") == 1
#$ assert-c-count: contains("ember_weak_release(") == 1
#$ assert-c: contains("ember_weak_upgrade(")

# The omitted parameter mode is borrowed. Passing a weakref T field to
# this function therefore reuses the field's link: the class field is
# its only weak owner, and no retain/release pair appears at the call boundary.
struct Token:
    value: i32

class Holder:
    weak: weakref Token

fn observe(weak: weakref Token) -> i32:
    if weak exists as owner:
        return owner.value
    else:
        return 0

fn main():
    strong = shares mut Token(7)
    holder = Holder(weakref strong)
    println(observe(holder.weak))
