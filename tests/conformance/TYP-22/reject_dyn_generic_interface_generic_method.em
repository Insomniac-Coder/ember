#$ test: compile-fail
#$ profiles: debug, development, release
#$ rules: TYP-16, TYP-22

interface Generic[T]:
    [U]
    fn convert(value: U):
        pass

fn take(x: ref dyn Generic[i32]):    #$ error[E2050]: `Generic[i32]` cannot be used as `dyn Generic[i32]`: `convert` is generic, and a `dyn` value has one entry per method
    pass
