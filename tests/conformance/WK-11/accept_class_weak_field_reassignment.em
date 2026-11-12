#$ test: run-pass
#$ rules: OBJ-3, HEAP-7, WK-11, WK-12, WK-14, TST-26
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# A class field may replace its empty weakref C handle with a live one. The
# assignment owns the new link and releases the old empty handle; it
# must not retain the observed class strongly.
class Token:
    value: i32

class Holder:
    weak: weakref Token = None

fn main():
    strong = Token(42)
    holder = Holder()
    holder.weak = weakref strong
    if holder.weak exists as owner:
        println(owner.value)
    else:
        println(0)
