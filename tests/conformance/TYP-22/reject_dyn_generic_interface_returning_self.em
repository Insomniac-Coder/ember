#$ test: compile-fail
#$ profiles: debug, development, release
#$ rules: TYP-16, TYP-22

interface Clone[T]:
    fn clone() -> Self:
        todo()

fn take(x: ref dyn Clone[i32]):    #$ error[E2050]: `Clone[i32]` cannot be used as `dyn Clone[i32]`: `clone` returns `Self` by value, and through a `dyn Clone[i32]` the kind of the result is unknown
    pass
