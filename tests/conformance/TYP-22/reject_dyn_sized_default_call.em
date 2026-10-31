#$ test: compile-fail
#$ profiles: debug, development, release
#$ rules: TYP-22

interface Factory:
    fn clone() -> Self where Self: needs Sized:
        todo()

fn take(x: ref dyn Factory):
    x.clone() #$ error[E2020]: requires `Self: Sized`

fn main():
    pass
