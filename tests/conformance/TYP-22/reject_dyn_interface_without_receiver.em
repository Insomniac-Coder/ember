#$ test: compile-fail
#$ rules: TYP-22

interface Static:
    fn make() -> i32:
        return 1

fn take(x: ref dyn Static):    #$ error[E2050]: method `make` has no receiver
    pass

