#$ test: compile-fail
#$ rules: TYP-22

interface Clone:
    fn clone() -> Self:
        todo()

fn take(x: ref dyn Clone):    #$ error[E2050]: method `clone` returns `Self` by value
    pass

