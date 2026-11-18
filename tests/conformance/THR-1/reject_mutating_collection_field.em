#$ test: compile-fail
#$ rules: THR-1, CLS-8
#$ profiles: debug
# `[THR-1]` — a list in a `thread_shared` value is filled where the value is made, or kept under a lock.

class Token:
    items: Array[int]

    fn init():
        this.items = [1]

fn main():
    token = thread_shared Token()
    token.items.push(owns 2)    #$ error[E7014]: is shared between threads
