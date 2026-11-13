#$ test: compile-fail
#$ rules: FFI-26, TIER-1
# `[FFI-26]`, `[TIER-1]` — only the two export attributes have an unsafe form: no other attribute
# introduces an `unsafe` boundary.

# SAFETY: none
@unsafe(derive(Copy))    #$ error[E0104]: `@derive` has no unsafe form: only `@unsafe(export(…))` and `@unsafe(export_table(…))` are written so
struct Point:
    x: i32

fn main():
    pass
