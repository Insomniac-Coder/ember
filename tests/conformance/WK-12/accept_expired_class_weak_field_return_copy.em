#$ test: run-pass
#$ rules: OBJ-3, HEAP-7, WK-11, WK-12, WK-14, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# A Weak[C] returned from a class field remains a valid but expired observer
# once both the source Holder and its final Token owner leave the helper. Its
# control block survives solely for the returned weak handle; upgrade is None.
class Token:
    value: i32

class Holder:
    weak: Weak[Token]

fn expired() -> Weak[Token]:
    strong = Token(7)
    holder = Holder(Weak(strong))
    return holder.weak

fn main():
    weak = expired()
    match weak.upgrade():
        Some(_):
            println(0)
        None:
            println(7)
