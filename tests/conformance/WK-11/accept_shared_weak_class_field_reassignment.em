#$ test: run-pass
#$ rules: HEAP-3, HEAP-4, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# A class field may replace an empty weakref T value with an observer of
# a live `shares` allocation. The field owns only a weak count; payload access
# after upgrade still crosses the explicit the owner read as its value boundary.
struct Token:
    value: i32

class Holder:
    weak: weakref Token = None

fn main():
    strong = shares mut Token(42)
    holder = Holder()
    holder.weak = weakref strong
    if holder.weak exists as owner:
        println(owner.value)
    else:
        println(0)
