#$ test: compile-pass
#$ rules: UNS-7
#$ profiles: debug

@safety("The caller keeps the input valid.")
pub unsafe fn operation(x: i32) -> i32:  # SAFETY: this test sets up what the operation requires
    return x

fn main():
    pass
