#$ test: run-pass
#$ rules: WK-5, WK-6, WK-7, WK-10, OBJ-3, WK-11, WK-12, TST-14
#$ profiles: debug, release, shipping
#$ stdout: 7

# `Holder[Child]` remains a strong instantiated edge, while the closing
# `Child.parent` field is weak. Substitution must preserve that distinction.
class Holder[T]:
    value: T

class Parent:
    holder: Holder[Child]

class Child:
    value: i32 = 7
    parent: Weak[Parent] = Weak[Parent].empty()

fn main():
    child = Child()
    println(child.value)
