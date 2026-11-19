#$ test: run-pass
#$ rules: CLS-4, HEAP-3, HEAP-4, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, development, release
#$ stdout: 7
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# Monomorphizing a generic class must retain its weakref T field's normal
# field-drop glue. The unrelated generic marker must not change the weak
# observer's ownership or the explicit the owner read as its value payload boundary.
struct Token:
    value: i32

[T]
class Holder:
    weak: weakref Token
    marker: T

fn main():
    strong = shares mut Token(7)
    holder = Holder[bool](weakref strong, true)
    if holder.weak exists as owner:
        println(owner.value)
    else:
        println(0)
