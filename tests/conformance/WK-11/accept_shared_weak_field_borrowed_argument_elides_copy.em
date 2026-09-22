#$ test: run-pass
#$ rules: FN-1, HEAP-3, HEAP-4, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ assert-c-count: contains("ember_weak_retain(") == 1
#$ assert-c-count: contains("ember_weak_release(") == 1
#$ assert-c: contains("ember_weak_upgrade(")

# The omitted parameter mode is borrowed. Passing a Weak[Shared[T]] field to
# this function therefore reuses the field's weak handle: the class field is
# its only weak owner, and no retain/release pair appears at the call boundary.
struct Token:
    value: i32

class Holder:
    weak: Weak[Shared[Token]]

fn observe(weak: Weak[Shared[Token]]) -> i32:
    match weak.upgrade():
        Some(owner):
            return owner.get().value
        None:
            return 0

fn main():
    strong = Shared(Token(7))
    holder = Holder(Weak(strong))
    println(observe(holder.weak))
