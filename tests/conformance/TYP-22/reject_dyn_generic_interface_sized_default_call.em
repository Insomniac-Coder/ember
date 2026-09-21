#$ test: compile-fail
#$ profiles: debug, release, shipping
#$ rules: TYP-16, TYP-22

interface Factory[T]:
    fn clone(self) -> Self where Self: Sized:
        pass

fn take(x: ref dyn Factory[i32]):
    x.clone() #$ error[E2020]: requires `Self: Sized`

fn main():
    pass
