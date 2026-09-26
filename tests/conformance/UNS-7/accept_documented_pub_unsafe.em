#$ test: compile-pass
#$ rules: UNS-7
#$ profiles: debug

@safety("The caller keeps the input valid.")
pub unsafe fn operation(x: i32) -> i32:
    return x

fn main():
    pass
