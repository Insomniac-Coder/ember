#$ test: compile-fail
#$ rules: STA-1, FFI-10
#$ error[E7002]: foreign static type `*i32` is not Sync
# `[STA-1]` — a static every thread may read without `unsafe` holds a `Sync` value. A foreign
# static marked `@ffi(immutable)` is read without `unsafe` (Part XVI's table), so it must be
# `Sync`, and a raw pointer is not.

unsafe extern "C":
    @ffi(immutable)
    static foreign_pointer: *i32

fn main():
    pass
