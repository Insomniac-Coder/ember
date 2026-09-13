#$ test: compile-fail
#$ rules: TYP-17

from std.core import Default

fn make[T]() -> T:
    return T.default() #$ error[E2040]: `T` has no associated function `default`; its bounds do not provide one

fn main():
    pass
