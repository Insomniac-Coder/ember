#$ test: compile-fail
#$ rules: FFI-25
#$ profiles: debug
#$ error[E0104]: `on_panic` in `@export` must be `abort`
# `[FFI-25]` — `@export(on_panic=abort)` is the only form in this version: another policy is
# refused as not admitted, never as something still to come.

@export(on_panic=unwind)
pub fn unwind_not_supported() -> i32:
    return 0

fn main():
    pass
