#$ test: run-pass
#$ rules: OBJ-3, HEAP-7, WK-11, WK-12, WK-14, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# Weak[C] has the same explicit owned-copy boundary as Weak[Shared[T]], while
# retaining its distinct class-handle representation. Copying a field into the
# owned parameter gives the callee its own weak handle, not a strong Token.
class Token:
    value: i32

class Holder:
    weak: Weak[Token]

fn observe(owned weak: Weak[Token]) -> i32:
    match weak.upgrade():
        Some(owner):
            return owner.value
        None:
            return 0

fn main():
    strong = Token(7)
    holder = Holder(Weak(strong))
    println(observe(holder.weak))
