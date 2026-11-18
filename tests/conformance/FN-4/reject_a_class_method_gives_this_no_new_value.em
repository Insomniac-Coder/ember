#$ test: compile-fail
#$ rules: FN-4, CLS-7
# `[FN-4]` — a class's method may not give `this` a whole new value (`[CLS-7]`).

class Cell2:
    n: int

    fn clear():
        this = Cell2(n=0)    #$ error[E2103]: cannot assign to `this` in a class method
