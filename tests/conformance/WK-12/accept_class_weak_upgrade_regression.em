#$ test: run-pass
#$ rules: WK-11, WK-12, WK-14, HEAP-7, TST-26, OBJ-3
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ stdout: end
#$ assert-c: contains(ember_weak_retain)
#$ assert-c: contains(ember_weak_upgrade)
#$ assert-c: contains(ember_weak_release)

class Token:
    value: i32

# `program` holds the test, so its values die before `main`'s last
# statement: at the end of `main` a release build leaves what only frees
# memory to the operating system (`[PHIL-5]`), and the drops read here
# would go with it.
fn program():
    strong = Token(42)
    weak: Weak[Token] = Weak(strong)
    match weak.upgrade():
        Some(owner):
            println(owner.value)
        None:
            println(0)

fn main():
    program()
    println("end")
