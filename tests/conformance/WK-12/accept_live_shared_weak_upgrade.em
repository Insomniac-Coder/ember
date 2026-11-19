#$ test: run-pass
#$ rules: HEAP-3, HEAP-4, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, development, release
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c-count: contains("ember_weak_retain(") == 1
#$ stdout: 7

# A live weakref T upgrades to another strong owner of the same control
# block. The payload remains behind `shares`'s explicit get() boundary.
struct Token:
    value: i32

fn main():
    owner = shares mut Token(7)
    weak: weakref Token = weakref owner
    if weak exists as live:
        value = live
        println(value.value)
    else:
        println(0)
