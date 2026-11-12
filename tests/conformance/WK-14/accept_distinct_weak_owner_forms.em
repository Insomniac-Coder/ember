#$ test: compile-pass
#$ rules: WK-14, HEAP-7
# `weakref C` and `weakref T` coexist as distinct, explicitly typed
# owner forms. The paired rejection test verifies that neither converts to
# the other.

struct Token:
    value: i32

class ClassOwner:
    value: i32

fn main():
    class_owner = ClassOwner(1)
    _class_weak: weakref ClassOwner = weakref class_owner
    shared = shares mut Token(2)
    _shared_weak: weakref Token = weakref shared
