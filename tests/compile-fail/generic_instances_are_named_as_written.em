#$ test: compile-fail
#$ rules: DIA-2
# `[DIA-2]` (D-232) — a diagnostic names a generic instance as it is written,
# `Wrap[i64]`, never by the compiler's name for the instance.

@no_derive(Debug)
struct Wrap[T]:
    value: T

fn main():
    w = Wrap(value=1)
    println(w)              #$ error[E2040]: `Wrap[i64]` does not implement `Debug`
    n: int = w              #$ error[E2020]: expected `i64`, found `Wrap[i64]`
    c = Cell(1)
    m: int = c              #$ error[E2020]: found `Cell[i64]`
