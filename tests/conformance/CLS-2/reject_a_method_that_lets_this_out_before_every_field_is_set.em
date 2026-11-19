#$ test: compile-fail
#$ rules: CLS-2
# `[CLS-2]` — a method whose body lets `this` out is called only after every field is set.

class Player:
    id: int
    score: int

    fn init():
        this.id = 1
        _l = this.link()    #$ error[E2100]
        this.score = 0

    fn link() -> weakref Player:
        return weakref this

fn main():
    pass
