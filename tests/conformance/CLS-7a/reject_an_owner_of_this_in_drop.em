#$ test: compile-fail
#$ rules: CLS-7a, OBJ-5
# `[CLS-7a]` — no owner of `this` is made inside `drop`: the object has already ended.

class Node:
    v: int = 0

    fn drop():
        _o = thread_shared this    #$ error[E3016]

fn main():
    _n = Node()
