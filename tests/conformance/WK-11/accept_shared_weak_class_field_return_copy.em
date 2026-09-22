#$ test: run-pass
#$ rules: HEAP-3, HEAP-4, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# Returning a Weak[Shared[T]] read from a class field gives the caller its own
# weak handle before the borrowed Holder parameter ends. The returned observer
# may upgrade while the independent strong owner remains live.
struct Token:
    value: i32

class Holder:
    weak: Weak[Shared[Token]]

fn observe(holder: Holder) -> Weak[Shared[Token]]:
    return holder.weak

fn main():
    strong = Shared(Token(7))
    holder = Holder(Weak(strong))
    weak = observe(holder)
    match weak.upgrade():
        Some(owner):
            println(owner.get().value)
        None:
            println(0)
