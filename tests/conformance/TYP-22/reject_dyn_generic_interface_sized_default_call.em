#$ test: compile-fail
#$ profiles: debug, development, release
#$ rules: TYP-16, TYP-22

interface Factory[T]:
    fn clone() -> Self where Self: needs Sized:
        todo()

fn take(x: ref dyn Factory[i32]):
    x.clone() #$ error[E2020]: requires `Self: Sized`

fn main():
    pass
