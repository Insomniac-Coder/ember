#$ test: compile-pass
#$ rules: UNS-7
#$ profiles: debug
#$ warning[L3016]: `@safety` text still reads `TODO`

@safety("TODO: state the caller's obligation")
pub unsafe fn operation(x: i32) -> i32:
    return x

fn main():
    pass
