#$ test: run-pass
#$ rules: HEAP-3, HEAP-4, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26, OBJ-3
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ stdout: 7
#$ stdout: 42
#$ assert-c: contains(ember_retain((ember_obj_header*)
#$ assert-c: contains(ember_release((ember_obj_header*)
#$ assert-c: contains(ember_weak_retain)
#$ assert-c: contains(ember_weak_upgrade)

struct Token:
    value: i32

    fn drop(mut self):
        println(self.value)

fn expired_copy() -> Weak[Shared[Token]]:
    first = Shared(Token(7))
    second = first
    observed = second.get()
    println(observed.value)
    weak = Weak(second)
    copy = weak
    return copy

fn main():
    stale = expired_copy()
    match stale.upgrade():
        Some(_):
            println(0)
        None:
            println(42)
