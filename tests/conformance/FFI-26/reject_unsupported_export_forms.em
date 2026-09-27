#$ test: compile-fail
#$ rules: FFI-26, ATT-6
#$ profiles: debug
#$ error[E0900]: `threads=creator` is not implemented for exported functions
#$ error[E0900]: an export name that is not a C identifier is not implemented yet

@export(threads=creator)
fn threaded() -> i32:
    return 0

@export("not-C")
fn odd_name() -> i32:
    return 1

fn main():
    pass
