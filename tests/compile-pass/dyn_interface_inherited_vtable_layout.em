#$ test: compile-pass
#$ rules: TYP-22
#$ assert-c: contains(int32_t (*slot0)(void*))
#$ assert-c: contains(int32_t (*slot1)(void*))

interface Parent:
    fn parent(self) -> i32

interface Child: Parent:
    fn child(self) -> i32

fn take(x: ref dyn Child) -> i32:
    return x.child()

fn main():
    pass
