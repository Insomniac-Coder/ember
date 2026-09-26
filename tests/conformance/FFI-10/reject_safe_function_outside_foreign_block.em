#$ test: compile-fail
#$ rules: FFI-10
#$ profiles: debug
#$ error[E0104]: `safe fn` is only valid inside `unsafe extern` declarations

safe fn answer() -> i32:
    return 42

fn main():
    println(answer())
