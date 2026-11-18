#$ test: compile-fail
#$ rules: THR-1, CLS-8
#$ profiles: debug
# `[THR-1]` — no owner writes a field of a `thread_shared` value.

class Token:
    value: int

    fn init():
        this.value = 1

fn main():
    token = thread_shared Token()
    token.value = 2    #$ error[E7014]: is shared between threads
