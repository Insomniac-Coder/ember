#$ test: compile-fail
#$ rules: THR-1
#$ profiles: debug
# `[THR-1]` — `ref mut` of a field of a `thread_shared` value would change it directly.

class Token:
    value: int

    fn init():
        this.value = 1

fn main():
    token = thread_shared Token()
    _borrow = ref mut token.value    #$ error[E7014]: is shared between threads
