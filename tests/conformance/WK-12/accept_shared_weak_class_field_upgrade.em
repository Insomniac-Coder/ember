#$ test: run-pass
#$ rules: HEAP-3, HEAP-4, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ assert-c: contains("ember_weak_upgrade(")

# A Weak[Shared[T]] may live in a class field. Reading that field gives a
# borrowed weak handle; upgrading it while the separate strong owner is live
# returns the same Shared allocation.
struct Token:
    value: i32

class Holder:
    weak: Weak[Shared[Token]]

fn main():
    owner = Shared(Token(7))
    holder = Holder(Weak(owner))
    match holder.weak.upgrade():
        Some(live):
            println(live.get().value)
        None:
            println(0)
