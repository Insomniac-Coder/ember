#$ test: compile-fail
#$ rules: SOA-1
#$ error[E2071]: `SoA` holds a struct's fields in columns; `i64` is not a struct
# `[SOA-1]` — `SoA[T]` keeps one column per field of the struct `T`.

fn main():
    bad = SoA[int].with_capacity(4)
