#$ test: compile-fail
#$ rules: CLS-2
# `[CLS-2]` — making a link or an owner of `this` uses it as a whole.

class Linked:
    a: int
    b: int

    fn init():
        this.a = 1
        _w = weakref this          #$ error[E2100]
        this.b = 2

class Owned:
    a: int
    b: int

    fn init():
        this.a = 1
        _o = thread_shared this    #$ error[E2100]
        this.b = 2

fn main():
    pass
