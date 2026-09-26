#$ test: compile-pass
#$ rules: UNS-7
#$ profiles: debug
#$ warning[L3015]: undocumented unsafe obligation

pub unsafe fn operation(x: i32) -> i32:
    return x

fn main():
    pass
