#$ test: compile-fail
#$ rules: TYP-22

interface Clone:
    fn clone() -> Self:
        todo()

fn take(x: ref dyn Clone):    #$ error[E2050]: `Clone` cannot be used as `dyn Clone`: `clone` returns `Self` by value, and through a `dyn Clone` the kind of the result is unknown
    pass

