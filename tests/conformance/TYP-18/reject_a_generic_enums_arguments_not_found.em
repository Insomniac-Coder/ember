#$ test: compile-fail
#$ rules: TYP-18, ENM-1
# `[TYP-18]` — a variant without a payload says nothing of its enum's
# arguments; with no type expected either, the diagnostic names the fixes.

enum Maybe[T]:
    Nothing
    Just(T)

fn main():
    _m = Maybe.Nothing #$ error[E2020]: cannot tell `Maybe`'s `T` from `Maybe.Nothing`
