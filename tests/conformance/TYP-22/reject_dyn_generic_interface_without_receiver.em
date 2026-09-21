#$ test: compile-fail
#$ profiles: debug, release, shipping
#$ rules: TYP-16, TYP-22

interface Static[T]:
    fn make() -> i32: #$ error[E2050]: method `make` has no receiver
        return 1

fn take(x: ref dyn Static[i32]):
    pass
