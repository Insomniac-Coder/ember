#$ test: run-pass
#$ rules: HEAP-3, HEAP-4, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c-count: contains("ember_weak_retain(") == 1
#$ stdout: 7

# A live Weak[Shared[T]] upgrades to another strong owner of the same control
# block. The payload remains behind Shared's explicit get() boundary.
struct Token:
    value: i32

fn main():
    owner = Shared(Token(7))
    weak: Weak[Shared[Token]] = Weak(owner)
    match weak.upgrade():
        Some(live):
            value = live.get()
            println(value.value)
        None:
            println(0)
