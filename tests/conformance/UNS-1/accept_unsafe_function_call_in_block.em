#$ test: run-pass
#$ rules: UNS-1, TIER-1
#$ profiles: debug, release, shipping
#$ stdout: 42

unsafe fn increase(x: i32) -> i32:
    return x + 1

fn main():
    unsafe:
        println(increase(41))
