#$ test: compile-fail
#$ rules: TYP-22

interface Generic:
    [T]
    fn convert(value: T):
        pass

fn take(x: ref dyn Generic):    #$ error[E2050]: `Generic` cannot be used as `dyn Generic`: `convert` is generic, and a `dyn` value has one entry per method
    pass

