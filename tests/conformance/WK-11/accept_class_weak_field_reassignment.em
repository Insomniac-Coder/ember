#$ test: run-pass
#$ rules: OBJ-3, HEAP-7, WK-11, WK-12, WK-14, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# A class field may replace its empty Weak[C] handle with a live one. The
# assignment owns the new weak handle and releases the old empty handle; it
# must not retain the observed class strongly.
class Token:
    value: i32

class Holder:
    weak: Weak[Token] = Weak[Token].empty()

fn main():
    strong = Token(42)
    holder = Holder()
    holder.weak = Weak(strong)
    match holder.weak.upgrade():
        Some(owner):
            println(owner.value)
        None:
            println(0)
