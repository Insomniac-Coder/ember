#$ test: compile-fail
#$ profiles: debug, release, shipping
#$ rules: TYP-16, TYP-22

interface Generic[T]:
    fn convert[U](self, value: U): #$ error[E2050]: method `convert` is generic
        pass

fn take(x: ref dyn Generic[i32]):
    pass
