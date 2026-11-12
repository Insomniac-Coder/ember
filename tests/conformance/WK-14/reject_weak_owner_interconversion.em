#$ test: compile-fail
#$ rules: WK-14, HEAP-7

struct Token:
    value: i32

class ClassOwner:
    value: i32

fn main():
    class_owner = ClassOwner(1)
    class_weak = weakref class_owner
    shared = shares mut Token(2)
    shared_weak = weakref shared
    class_weak = shared_weak    #$ error[E2020]: expected
