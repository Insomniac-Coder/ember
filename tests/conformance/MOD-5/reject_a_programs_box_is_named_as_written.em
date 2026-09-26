#$ test: compile-fail
#$ rules: MOD-5, DIA-2
# `[MOD-5]`, `[DIA-2]` — a program's own `Box` is kept apart from the prelude's
# inside the compiler (D-305), and a diagnostic still writes it as the program
# does: `Box[i64]`.

struct Box[T]:
    item: T

fn main():
    _b: Box[int] = 5 #$ error[E2020]: expected `Box[i64]`, found `an integer`
