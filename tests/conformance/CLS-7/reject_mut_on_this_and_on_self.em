#$ test: compile-fail
#$ rules: CLS-7
#$ help: fn reset():
#$ help: fn clear():
# `[CLS-7]` — a method changes its object through `this` with no word.

class Node:
    v: int = 0

    fn reset(mut this):    #$ error[E0113]
        this.v = 0

    fn clear(mut self):    #$ error[E1010]
        pass

fn main():
    pass
