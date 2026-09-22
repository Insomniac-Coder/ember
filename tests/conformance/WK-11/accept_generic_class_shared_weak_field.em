#$ test: run-pass
#$ rules: CLS-4, HEAP-3, HEAP-4, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# Monomorphizing a generic class must retain its Weak[Shared[T]] field's normal
# field-drop glue. The unrelated generic marker must not change the weak
# observer's ownership or the explicit Shared.get() payload boundary.
struct Token:
    value: i32

class Holder[T]:
    weak: Weak[Shared[Token]]
    marker: T

fn main():
    strong = Shared(Token(7))
    holder = Holder[bool](Weak(strong), true)
    match holder.weak.upgrade():
        Some(owner):
            println(owner.get().value)
        None:
            println(0)
