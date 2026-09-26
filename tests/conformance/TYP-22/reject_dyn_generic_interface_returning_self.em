#$ test: compile-fail
#$ profiles: debug, release, shipping
#$ rules: TYP-16, TYP-22

interface Clone[T]:
    fn clone(self) -> Self:
        todo()

fn take(x: ref dyn Clone[i32]):    #$ error[E2050]: method `clone` returns `Self` by value
    pass
