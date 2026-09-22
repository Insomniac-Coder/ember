#$ test: run-pass
#$ rules: HEAP-3, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26, OBJ-3
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# A Weak[Shared[T]] returned from a class field remains a valid observer after
# both its source Holder and the last Shared owner leave scope. It retains only
# the control block, so upgrade must report None and never resurrect Token.
struct Token:
    value: i32

class Holder:
    weak: Weak[Shared[Token]]

fn expired() -> Weak[Shared[Token]]:
    strong = Shared(Token(7))
    holder = Holder(Weak(strong))
    return holder.weak

fn main():
    weak = expired()
    match weak.upgrade():
        Some(_):
            println(0)
        None:
            println(7)
