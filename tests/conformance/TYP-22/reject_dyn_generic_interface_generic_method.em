#$ test: compile-fail
#$ profiles: debug, development, release
#$ rules: TYP-16, TYP-22

interface Generic[T]:
    fn convert[U](self, value: U):
        pass

fn take(x: ref dyn Generic[i32]):    #$ error[E2050]: method `convert` is generic
    pass
