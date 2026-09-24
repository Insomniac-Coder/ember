#$ test: compile-fail
#$ rules: TYP-22

interface Generic:
    fn convert[T](self, value: T):
        pass

fn take(x: ref dyn Generic):    #$ error[E2050]: method `convert` is generic
    pass

