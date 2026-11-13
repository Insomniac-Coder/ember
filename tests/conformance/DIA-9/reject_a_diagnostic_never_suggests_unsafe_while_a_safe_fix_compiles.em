#$ test: compile-fail
#$ rules: DIA-9, BRW-14
#$ error[E3023]: `values.push(owns 1)` changes the caller's list, but `values` only reads it; write `fn append_one(ref mut values: Array[i32]):` and call it `append_one(ref mut values)`
#$ not-help: unsafe
#$ not-help: shares
# `[DIA-9]` — a diagnostic never suggests `unsafe` while a fix without it compiles, and never
# suggests `shares` or `shares mut` alone where a fix that keeps one owner compiles.

fn append_one(values: Array[i32]):
    values.push(owns 1)

fn main():
    values: Array[i32] = Array[i32]()
    append_one(values)
