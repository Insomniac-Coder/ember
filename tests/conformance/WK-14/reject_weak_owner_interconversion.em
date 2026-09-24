#$ test: compile-fail
#$ rules: WK-14, HEAP-7

struct Token:
    value: i32

class ClassOwner:
    value: i32

fn main():
    class_owner = ClassOwner(1)
    class_weak = Weak(class_owner)
    shared = Shared(Token(2))
    shared_weak = Weak(shared)
    class_weak = shared_weak    #$ error[E2020]: expected
