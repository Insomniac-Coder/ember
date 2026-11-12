#$ test: run-pass
#$ rules: HEAP-3, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, WK-14, OBJ-3
#$ profiles: debug, development, release
#$ stdout: 1
#$ stdout: 42
#$ stdout: 2
#$ stdout: end
#$ assert-c: contains(ember_obj_new_copy)
#$ assert-c: contains(ember_weak_retain)
#$ assert-c: contains(ember_weak_upgrade)
#$ assert-c: contains(ember_weak_release)

struct Token:
    value: i32

fn expired() -> weakref Token:
    shared = shares mut Token(7)
    return weakref shared

# `program` holds the test, so its values die before `main`'s last
# statement: at the end of `main` a release build leaves what only frees
# memory to the operating system (`[PHIL-5]`), and the drops read here
# would go with it.
fn program():
    empty: weakref Token = None
    if empty exists as _:
        println(0)
    else:
        println(1)

    shared = shares mut Token(42)
    weak = weakref shared
    copy = weak
    if copy exists as owner:
        value = owner
        println(value.value)
    else:
        println(0)

    stale = expired()
    if stale exists as _:
        println(0)
    else:
        println(2)

fn main():
    program()
    println("end")
