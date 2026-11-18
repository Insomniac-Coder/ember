#$ test: compile-fail
#$ rules: FN-4
#$ not-help: build the
# `[FN-4]` — the help that builds the value is given only where the function returns its own type
# and sets each field it names through `this` from its parameters.

struct Counter:
    n: int

    static fn make(k: int) -> int:
        return this.n + k    #$ error[E1010]: `this` can only be used in a method, and `make` is a `static fn`, not a method
