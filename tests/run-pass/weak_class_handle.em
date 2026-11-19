#$ test: run-pass
#$ rules: OBJ-3, WK-2, WK-3
#$ profiles: debug, development, release
#$ stdout: 42
#$ stdout: 42
#$ stdout: 7
#$ stdout: 42
#$ stdout: 7
#$ assert-c: contains(ember_weak_retain)
#$ assert-c: contains(ember_weak_upgrade)
#$ assert-c: contains(ember_weak_release)

class Token:
    value: i32

    fn drop():
        println(7)

fn expired() -> weakref Token:
    token = Token(42)
    return weakref token

fn main():
    empty: weakref Token = None
    if empty exists as _:
        println(0)
    else:
        println(42)

    token = Token(42)
    weak = weakref token
    copy = weak
    if copy exists as value:
        if value.value == 0:
            println(0)
        else:
            println(value.value)
    else:
        println(0)

    stale = expired()
    if stale exists as _:
        println(0)
    else:
        println(42)
