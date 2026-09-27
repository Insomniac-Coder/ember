#$ test: compile-fail
#$ rules: FFI-33, ATT-6
#$ profiles: debug
#$ error[E0900]: `threads=creator` is not implemented for exported functions

@export(threads=creator)
fn creator_bound() -> i32:
    return 0

fn main():
    pass
