#$ test: compile-fail
#$ rules: FFI-26, ATT-6
#$ profiles: debug
#$ error[E0900]: only `@export("C_identifier")` and `on_panic=abort` are implemented yet
#$ error[E0900]: an export name that is not a C identifier is not implemented yet

@export(threads=main)
fn threaded() -> i32:
    return 0

@export("not-C")
fn odd_name() -> i32:
    return 1

fn main():
    pass
