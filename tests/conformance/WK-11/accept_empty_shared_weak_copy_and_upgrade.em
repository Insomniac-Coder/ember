#$ test: run-pass
#$ rules: HEAP-3, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_weak_retain(") == 1
#$ assert-c-count: contains("ember_weak_release(") == 2
#$ stdout: 42
#$ stdout: end

# An empty Weak[Shared[T]] remains empty when copied. The copy is a weak-handle
# operation, never a construction of a Shared control block or strong owner.
struct Token:
    value: i32

# `program` holds the test, so its values die before `main`'s last
# statement: at the end of `main` a release build leaves what only frees
# memory to the operating system (`[PHIL-5]`), and the drops read here
# would go with it.
fn program():
    empty: Weak[Shared[Token]] = Weak[Shared[Token]].empty()
    copy = empty
    match copy.upgrade():
        Some(_):
            println(0)
        None:
            println(42)

fn main():
    program()
    println("end")
