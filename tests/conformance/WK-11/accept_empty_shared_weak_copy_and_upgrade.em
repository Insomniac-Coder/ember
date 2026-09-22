#$ test: run-pass
#$ rules: HEAP-3, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_weak_retain(") == 1
#$ assert-c-count: contains("ember_weak_release(") == 2
#$ stdout: 42

# An empty Weak[Shared[T]] remains empty when copied. The copy is a weak-handle
# operation, never a construction of a Shared control block or strong owner.
struct Token:
    value: i32

fn main():
    empty: Weak[Shared[Token]] = Weak[Shared[Token]].empty()
    copy = empty
    match copy.upgrade():
        Some(_):
            println(0)
        None:
            println(42)
