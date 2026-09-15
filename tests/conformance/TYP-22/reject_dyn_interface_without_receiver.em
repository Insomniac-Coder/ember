#$ test: compile-fail
#$ rules: TYP-22

interface Static:
    fn make() -> i32: #$ error[E2050]: method `make` has no receiver
        return 1

fn take(x: ref dyn Static):
    pass

