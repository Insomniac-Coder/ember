#$ test: compile-pass
#$ rules: UNS-7
#$ profiles: debug
#$ warning[L3015]: `operation` is an `unsafe fn`, and nothing says what its callers must make true; say it above the function: `@safety("...")`

pub unsafe fn operation(x: i32) -> i32:
    return x

fn main():
    pass
