#$ test: compile-fail
#$ profiles: debug, release, shipping
#$ rules: TYP-22

interface Factory:
    fn clone(self) -> Self where Self: Sized:
        todo()

fn take(x: ref dyn Factory):
    x.clone() #$ error[E2020]: requires `Self: Sized`

fn main():
    pass
