#$ test: compile-fail
#$ rules: FFI-26, MNG-2
#$ profiles: debug
#$ error[E0104]: C export `main` conflicts with the generated program entry point

@export("main")
fn host_entry() -> i32:
    return 1

fn main():
    pass
