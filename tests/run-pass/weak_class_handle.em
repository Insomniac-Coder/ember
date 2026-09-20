#$ test: run-pass
#$ rules: OBJ-3, WK-2, WK-3
#$ profiles: debug, release, shipping
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

    fn drop(mut self):
        println(7)

fn expired() -> Weak[Token]:
    token = Token(42)
    return Weak(token)

fn main():
    empty = Weak[Token].empty()
    match empty.upgrade():
        Some(_) => println(0)
        None => println(42)

    token = Token(42)
    weak = Weak(token)
    copy = weak
    match copy.upgrade():
        Some(first) if first.value == 0 => println(0)
        Some(value) => println(value.value)
        None => println(0)

    stale = expired()
    match stale.upgrade():
        Some(_) => println(0)
        None => println(42)
