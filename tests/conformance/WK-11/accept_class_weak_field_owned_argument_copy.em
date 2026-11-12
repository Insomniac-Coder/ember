#$ test: run-pass
#$ rules: OBJ-3, HEAP-7, WK-11, WK-12, WK-14, TST-26
#$ profiles: debug, development, release
#$ stdout: 7
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# weakref C has the same explicit owned-copy boundary as weakref T, while
# retaining its distinct class-handle representation. Copying a field into the
# owned parameter gives the callee its own link, not a strong Token.
class Token:
    value: i32

class Holder:
    weak: weakref Token

fn observe(owns weak: weakref Token) -> i32:
    if weak exists as owner:
        return owner.value
    else:
        return 0

fn main():
    strong = Token(7)
    holder = Holder(weakref strong)
    println(observe(owns (weakref holder.weak)))
