#$ test: compile-fail
#$ rules: THR-1, EXC-15
#$ profiles: debug
# `[THR-1]` — no owner changes a `thread_shared` value directly: a method that changes its object is
# not called through one. What a method does to it is worked out from its body.

class Token:
    value: int = 1

    fn bump():
        this.value += 1

fn main():
    t = thread_shared Token()
    t.bump()    #$ error[E7014]: is shared between threads
