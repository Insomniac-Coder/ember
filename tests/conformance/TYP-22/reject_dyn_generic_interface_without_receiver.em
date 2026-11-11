#$ test: compile-fail
#$ profiles: debug, development, release
#$ rules: TYP-16, TYP-22

interface Static[T]:
    static fn make() -> i32:
        return 1

fn take(x: ref dyn Static[i32]):    #$ error[E2050]: `Static[i32]` cannot be used as `dyn Static[i32]`: `make` is a `static fn`, and a `dyn` value has no object to call it on
    pass
