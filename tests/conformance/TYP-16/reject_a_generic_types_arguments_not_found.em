#$ test: compile-fail
#$ rules: TYP-16
# `[TYP-16]` — when neither the arguments nor the expected type say what a
# generic struct's parameter is, the call names the fix: write the arguments.

struct Stack[T]:
    items: Array[T]

    fn empty() -> Stack[T]:
        return Stack(Array())

fn main():
    _s = Stack.empty() #$ error[E2020]: cannot tell `Stack`'s `T` from this call to `empty`
