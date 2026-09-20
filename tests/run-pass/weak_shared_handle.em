#$ test: run-pass
#$ rules: HEAP-3, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, WK-14, OBJ-3
#$ profiles: debug, release, shipping
#$ stdout: 1
#$ stdout: 42
#$ stdout: 2
#$ assert-c: contains(ember_obj_new_copy)
#$ assert-c: contains(ember_weak_retain)
#$ assert-c: contains(ember_weak_upgrade)
#$ assert-c: contains(ember_weak_release)

struct Token:
    value: i32

fn expired() -> Weak[Shared[Token]]:
    shared = Shared(Token(7))
    return Weak(shared)

fn main():
    empty: Weak[Shared[Token]] = Weak[Shared[Token]].empty()
    match empty.upgrade():
        Some(_):
            println(0)
        None:
            println(1)

    shared = Shared(Token(42))
    weak = Weak(shared)
    copy = weak
    match copy.upgrade():
        Some(owner):
            value = owner.get()
            println(value.value)
        None:
            println(0)

    stale = expired()
    match stale.upgrade():
        Some(_):
            println(0)
        None:
            println(2)
