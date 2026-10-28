#$ test: run-pass
#$ rules: UNS-1, TIER-1
#$ profiles: debug, development, release
#$ stdout: 42

@safety("this test sets up what the operation requires")
unsafe fn increase(x: i32) -> i32:
    return x + 1

fn main():
    unsafe:  # SAFETY: this test sets up what the operation requires
        println(increase(41))
