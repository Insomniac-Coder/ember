#$ test: run-pass
#$ rules: WK-11, WK-12, WK-14, HEAP-7, TST-26, OBJ-3
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(ember_weak_retain)
#$ assert-c: contains(ember_weak_upgrade)
#$ assert-c: contains(ember_weak_release)

class Token:
    value: i32

fn main():
    strong = Token(42)
    weak: Weak[Token] = Weak(strong)
    match weak.upgrade():
        Some(owner):
            println(owner.value)
        None:
            println(0)
