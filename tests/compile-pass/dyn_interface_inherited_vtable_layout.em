#$ test: compile-pass
#$ rules: TYP-22
#$ assert-c: contains(int32_t (*slot0)(void*))
#$ assert-c: contains(int32_t (*slot1)(void*))

interface Parent:
    fn parent() -> i32

interface Child: needs Parent:
    fn child() -> i32

fn take(x: ref dyn Child) -> i32:
    return x.child()

fn main():
    pass
