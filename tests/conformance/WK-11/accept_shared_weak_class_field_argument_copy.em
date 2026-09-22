#$ test: run-pass
#$ rules: HEAP-3, HEAP-4, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# Passing a Weak[Shared[T]] class field to an `owned` parameter copies its weak
# handle for the callee. The copied observer may upgrade while the caller's
# separate Shared owner is live, but it never turns the field read into a
# payload copy.
struct Token:
    value: i32

class Holder:
    weak: Weak[Shared[Token]]

fn observe(owned weak: Weak[Shared[Token]]) -> i32:
    match weak.upgrade():
        Some(owner):
            return owner.get().value
        None:
            return 0

fn main():
    strong = Shared(Token(7))
    holder = Holder(Weak(strong))
    println(observe(holder.weak))
